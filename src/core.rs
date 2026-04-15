use git2::{Repository, Signature, IndexAddOption, StatusOptions};
use std::path::Path;
use crate::error::{Result, ToriiError};

pub struct GitRepo {
    pub(crate) repo: Repository,
}

impl GitRepo {
    /// Initialize a new git repository
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = Repository::init(path)?;
        Ok(Self { repo })
    }

    /// Open an existing repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_ref = path.as_ref();
        let repo = Repository::discover(path_ref)
            .map_err(|_| ToriiError::RepositoryNotFound(
                path_ref.display().to_string()
            ))?;
        Ok(Self { repo })
    }

    /// Add all changes to staging
    pub fn add_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    /// Add specific files to staging
    pub fn add<P: AsRef<Path>>(&self, paths: &[P]) -> Result<()> {
        let mut index = self.repo.index()?;
        for path in paths {
            index.add_path(path.as_ref())?;
        }
        index.write()?;
        Ok(())
    }

    /// Commit changes
    pub fn commit(&self, message: &str) -> Result<()> {
        let sig = self.get_signature()?;
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        let parent_commit = self.repo.head()?.peel_to_commit()?;
        
        self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &[&parent_commit],
        )?;

        Ok(())
    }

    /// Amend the previous commit
    pub fn commit_amend(&self, message: &str) -> Result<()> {
        let sig = self.get_signature()?;
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        
        // Get the current HEAD commit
        let head_commit = self.repo.head()?.peel_to_commit()?;
        
        // Get the parents of the current commit (to preserve them)
        let parents: Vec<_> = head_commit.parents().collect();
        let parent_refs: Vec<_> = parents.iter().collect();
        
        // Amend by creating a new commit with the same parents
        self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &parent_refs,
        )?;
        
        Ok(())
    }

    /// Pull from remote
    pub fn pull(&self) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        
        // Configure SSH authentication
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                std::path::Path::new(&format!("{}/.ssh/id_ed25519", std::env::var("HOME").unwrap())),
                None,
            )
        });

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote.fetch(&["main", "master"], Some(&mut fetch_options), None)?;
        
        // Simple fast-forward merge
        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;
        
        let analysis = self.repo.merge_analysis(&[&fetch_commit])?;
        
        if analysis.0.is_up_to_date() {
            println!("Already up to date");
        } else if analysis.0.is_fast_forward() {
            let refname = "refs/heads/main";
            let mut reference = self.repo.find_reference(refname)?;
            reference.set_target(fetch_commit.id(), "Fast-forward")?;
            self.repo.set_head(refname)?;
            self.repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        }

        Ok(())
    }

    /// Push to remote
    pub fn push(&self, force: bool) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        let branch = self.get_current_branch()?;
        
        let refspec = if force {
            format!("+refs/heads/{}:refs/heads/{}", branch, branch)
        } else {
            format!("refs/heads/{}:refs/heads/{}", branch, branch)
        };

        // Configure SSH authentication
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key(
                username_from_url.unwrap(),
                None,
                std::path::Path::new(&format!("{}/.ssh/id_ed25519", std::env::var("HOME").unwrap())),
                None,
            )
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // Push branch
        remote.push(&[&refspec], Some(&mut push_options))?;

        // Push tags via git subprocess — git2 doesn't support glob refspecs
        // and remote object can't be reused after push without reconnecting
        let repo_path = self.repo.path().parent().unwrap();
        let mut tag_args = vec!["push", "origin", "--tags"];
        if force {
            tag_args.push("--force");
        }
        let tag_result = std::process::Command::new("git")
            .args(&tag_args)
            .current_dir(repo_path)
            .output();

        if let Ok(out) = tag_result {
            if !out.status.success() {
                let err = String::from_utf8_lossy(&out.stderr);
                eprintln!("⚠️  Tag push failed: {}", err.trim());
            }
        }

        Ok(())
    }

    /// Get current branch name
    pub fn get_current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        let branch_name = head.shorthand()
            .ok_or_else(|| ToriiError::Git(git2::Error::from_str("Could not get branch name")))?;
        Ok(branch_name.to_string())
    }

    /// Get the repository reference
    pub fn repository(&self) -> &Repository {
        &self.repo
    }

    /// Show repository status with context and suggestions
    pub fn status(&self) -> Result<()> {
        let mut opts = StatusOptions::new();
        opts.include_untracked(true);
        let statuses = self.repo.statuses(Some(&mut opts))?;

        // Header
        println!("📊 Repository Status\n");
        
        // Branch and commit info
        let branch = self.get_current_branch()?;
        println!("Branch: {}", branch);
        
        // Get latest commit info
        if let Ok(head) = self.repo.head() {
            if let Ok(commit) = head.peel_to_commit() {
                let msg = commit.message().unwrap_or("").lines().next().unwrap_or("");
                let time = commit.time();
                let timestamp = chrono::DateTime::from_timestamp(time.seconds(), 0)
                    .unwrap_or_default();
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(timestamp);
                
                let time_ago = if duration.num_days() > 0 {
                    format!("{} days ago", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{} hours ago", duration.num_hours())
                } else if duration.num_minutes() > 0 {
                    format!("{} minutes ago", duration.num_minutes())
                } else {
                    "just now".to_string()
                };
                
                let short_id = format!("{:.7}", commit.id());
                println!("Commit: {} - \"{}\" ({})", short_id, msg, time_ago);
            }
        }
        
        // Remote status
        if let Ok(remote) = self.repo.find_remote("origin") {
            if let Some(url) = remote.url() {
                let remote_name = url.split('/').last().unwrap_or("origin");
                print!("Remote: {}", remote_name.trim_end_matches(".git"));
                
                // Check if ahead/behind
                if let Ok(head) = self.repo.head() {
                    if let Ok(local_oid) = head.target().ok_or("No target") {
                        let remote_branch = format!("refs/remotes/origin/{}", branch);
                        if let Ok(remote_ref) = self.repo.find_reference(&remote_branch) {
                            if let Ok(remote_oid) = remote_ref.target().ok_or("No target") {
                                if let Ok((ahead, behind)) = self.repo.graph_ahead_behind(local_oid, remote_oid) {
                                    if ahead > 0 || behind > 0 {
                                        print!(" (");
                                        if ahead > 0 {
                                            print!("{} ahead", ahead);
                                        }
                                        if ahead > 0 && behind > 0 {
                                            print!(", ");
                                        }
                                        if behind > 0 {
                                            print!("{} behind", behind);
                                        }
                                        print!(")");
                                    } else {
                                        print!(" (up to date)");
                                    }
                                }
                            }
                        }
                    }
                }
                println!();
            }
        }
        
        println!();

        // Categorize changes
        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();

        for entry in statuses.iter() {
            let status = entry.status();
            let path = entry.path().unwrap_or("unknown").to_string();

            if status.is_index_new() || status.is_index_modified() || status.is_index_deleted() {
                let prefix = if status.is_index_new() {
                    "A "
                } else if status.is_index_modified() {
                    "M "
                } else {
                    "D "
                };
                staged.push(format!("{} {}", prefix, path));
            }
            
            if status.is_wt_modified() || status.is_wt_deleted() {
                let prefix = if status.is_wt_modified() {
                    "M "
                } else {
                    "D "
                };
                unstaged.push(format!("{} {}", prefix, path));
            }
            
            if status.is_wt_new() {
                untracked.push(format!("?? {}", path));
            }
        }

        // Show status
        let is_clean = staged.is_empty() && unstaged.is_empty() && untracked.is_empty();

        if is_clean {
            println!("✨ Working tree clean");
        } else {
            if !staged.is_empty() {
                println!("✅ Changes staged for commit:");
                for file in &staged {
                    println!("  {}", file);
                }
                println!();
            }

            if !unstaged.is_empty() {
                println!("📝 Changes not staged:");
                for file in &unstaged {
                    println!("  {}", file);
                }
                println!();
            }

            if !untracked.is_empty() {
                println!("📦 Untracked files:");
                for file in &untracked {
                    println!("  {}", file);
                }
                println!();
            }
        }

        // Next steps suggestions
        println!("💡 Next steps:");
        if is_clean {
            println!("  • Start new work: torii switch -c feature-name");
            println!("  • Update from remote: torii sync");
            println!("  • Create snapshot: torii snapshot create");
        } else if !staged.is_empty() && unstaged.is_empty() && untracked.is_empty() {
            println!("  • Commit staged changes: torii save -m \"message\"");
            println!("  • See staged changes: torii diff --staged");
            println!("  • Unstage changes: git reset HEAD");
        } else if !unstaged.is_empty() || !untracked.is_empty() {
            println!("  • Save all changes: torii save -am \"message\"");
            println!("  • See changes: torii diff");
            if !staged.is_empty() {
                println!("  • Commit only staged: torii save -m \"message\"");
            }
        }

        Ok(())
    }

    /// Get git signature from config or use defaults
    fn get_signature(&self) -> Result<Signature> {
        let config = self.repo.config()?;
        
        let name = config
            .get_string("user.name")
            .unwrap_or_else(|_| "Torii User".to_string());
        
        let email = config
            .get_string("user.email")
            .unwrap_or_else(|_| "user@torii.local".to_string());

        Ok(Signature::now(&name, &email)?)
    }
}
