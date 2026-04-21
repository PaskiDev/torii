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
        let git_repo = Self { repo };
        // Sync .toriignore on every open so all git operations respect it
        git_repo.sync_toriignore()?;
        Ok(git_repo)
    }

    /// Sync .toriignore → .git/info/exclude so git itself respects the patterns.
    /// Called automatically on open and before staging.
    pub fn sync_toriignore(&self) -> Result<()> {
        let repo_path = self.repo.path().parent().unwrap().to_path_buf();
        let toriignore_path = repo_path.join(".toriignore");
        let exclude_path = self.repo.path().join("info").join("exclude");

        if toriignore_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&toriignore_path) {
                let header = "# Synced from .toriignore by torii — do not edit manually\n";
                let _ = std::fs::write(&exclude_path, format!("{}{}", header, content));
            }
        }

        Ok(())
    }

    /// Add all changes to staging, respecting .toriignore
    pub fn add_all(&self) -> Result<()> {
        self.sync_toriignore()?;

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

        // Root commit (empty repo) has no parent
        let parent_commit = match self.repo.head() {
            Ok(head) => Some(head.peel_to_commit()?),
            Err(_) => None,
        };

        let parents: Vec<&git2::Commit> = parent_commit.iter().collect();

        self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            message,
            &tree,
            &parents,
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
    
    /// Build auth callbacks for SSH and HTTPS token auth.
    /// Pass the remote URL so the correct token is selected per host.
    pub fn auth_callbacks_for<'a>(url: &str) -> git2::RemoteCallbacks<'a> {
        let cfg = crate::config::ToriiConfig::load_global().unwrap_or_default();
        let url_owned = url.to_string();
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(move |cb_url, username_from_url, allowed_types| {
            let effective_url = if url_owned.is_empty() { cb_url } else { &url_owned };
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                let username = username_from_url.unwrap_or("git");
                let home = dirs::home_dir().unwrap_or_default();
                let ed25519 = home.join(".ssh").join("id_ed25519");
                let rsa = home.join(".ssh").join("id_rsa");
                if ed25519.exists() {
                    return git2::Cred::ssh_key(username, None, &ed25519, None);
                } else if rsa.exists() {
                    return git2::Cred::ssh_key(username, None, &rsa, None);
                } else {
                    return git2::Cred::ssh_key_from_agent(username);
                }
            }
            if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                let token = if effective_url.contains("github.com") {
                    cfg.auth.github_token.clone()
                } else if effective_url.contains("gitlab.com") {
                    cfg.auth.gitlab_token.clone()
                } else if effective_url.contains("codeberg.org") {
                    cfg.auth.codeberg_token.clone()
                } else {
                    cfg.auth.gitea_token.clone()
                };
                if let Some(token) = token {
                    return git2::Cred::userpass_plaintext("oauth2", &token);
                }
            }
            git2::Cred::default()
        });
        callbacks
    }

    /// Pull from remote
    pub fn pull(&self) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;

        let remote_url = remote.url().unwrap_or("").to_string();
        let callbacks = Self::auth_callbacks_for(&remote_url);

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote.fetch(&["main", "master"], Some(&mut fetch_options), None)?;
        
        // Simple fast-forward merge
        let fetch_head = self.repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = self.repo.reference_to_annotated_commit(&fetch_head)?;
        
        let analysis = self.repo.merge_analysis(&[&fetch_commit])?;
        
        if analysis.0.is_up_to_date() {
            // nothing to do
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

        let remote_url = remote.url().unwrap_or("").to_string();
        let callbacks = Self::auth_callbacks_for(&remote_url);

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        // Push branch
        remote.push(&[&refspec], Some(&mut push_options))?;

        // Push tags via git2 — enumerate local tags and push each one
        self.push_all_tags_via_git2("origin", force)?;

        Ok(())
    }

    /// Push all local tags to a remote using git2 (no subprocess needed)
    pub fn push_all_tags_via_git2(&self, remote_name: &str, force: bool) -> Result<()> {
        let tags = self.repo.tag_names(None)?;
        if tags.is_empty() {
            return Ok(());
        }
        let mut remote = self.repo.find_remote(remote_name)?;
        let remote_url = remote.url().unwrap_or("").to_string();
        let refspecs: Vec<String> = tags.iter()
            .flatten()
            .map(|t| {
                let r = format!("refs/tags/{}:refs/tags/{}", t, t);
                if force { format!("+{}", r) } else { r }
            })
            .collect();
        let refspec_refs: Vec<&str> = refspecs.iter().map(|s| s.as_str()).collect();
        if !refspec_refs.is_empty() {
            let callbacks = Self::auth_callbacks_for(&remote_url);
            let mut push_options = git2::PushOptions::new();
            push_options.remote_callbacks(callbacks);
            if let Err(e) = remote.push(&refspec_refs, Some(&mut push_options)) {
                eprintln!("⚠️  Tag push failed: {}", e);
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
            println!("  • Start new work: torii branch feature-name -c");
            println!("  • Update from remote: torii sync");
            println!("  • Create snapshot: torii snapshot create");
        } else if !staged.is_empty() && unstaged.is_empty() && untracked.is_empty() {
            println!("  • Commit staged changes: torii save -m \"message\"");
            println!("  • See staged changes: torii diff --staged");
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
    fn get_signature(&self) -> Result<Signature<'_>> {
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
