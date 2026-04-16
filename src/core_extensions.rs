// Extended Git operations for Torii
use git2::{Repository, BranchType};
use crate::error::Result;
use crate::core::GitRepo;
use chrono::{DateTime, NaiveDateTime};

impl GitRepo {
    /// Show commit history
    pub fn log(
        &self,
        count: Option<usize>,
        oneline: bool,
        _graph: bool,
        author: Option<&str>,
        since: Option<&str>,
        until: Option<&str>,
        grep: Option<&str>,
        stat: bool,
    ) -> Result<()> {
        let mut revwalk = self.repository().revwalk()?;
        revwalk.push_head()?;

        let max_count = count.unwrap_or(10);
        let mut shown = 0;

        // Parse date filters
        let since_ts: Option<i64> = since.and_then(|s| {
            NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.and_utc().timestamp())
        });
        let until_ts: Option<i64> = until.and_then(|s| {
            NaiveDateTime::parse_from_str(&format!("{} 23:59:59", s), "%Y-%m-%d %H:%M:%S")
                .ok()
                .map(|dt| dt.and_utc().timestamp())
        });

        println!("📜 Commit History:");
        println!();

        for oid in revwalk {
            if shown >= max_count {
                break;
            }

            let oid = oid?;
            let commit = self.repository().find_commit(oid)?;
            let ts = commit.time().seconds();

            // Author filter
            if let Some(filter) = author {
                let name = commit.author().name().unwrap_or("").to_lowercase();
                let email = commit.author().email().unwrap_or("").to_lowercase();
                let f = filter.to_lowercase();
                if !name.contains(&f) && !email.contains(&f) {
                    continue;
                }
            }

            // Date filters
            if let Some(s) = since_ts {
                if ts < s { continue; }
            }
            if let Some(u) = until_ts {
                if ts > u { continue; }
            }

            // Grep filter
            if let Some(pattern) = grep {
                let msg = commit.message().unwrap_or("");
                if !msg.to_lowercase().contains(&pattern.to_lowercase()) {
                    continue;
                }
            }

            if oneline {
                let short_id = &oid.to_string()[..7];
                let message = commit.message().unwrap_or("<no message>").lines().next().unwrap_or("");
                println!("  {} {}", short_id, message);
            } else {
                println!("  commit {}", oid);
                if let Some(author_name) = commit.author().name() {
                    println!("  Author: {}", author_name);
                }
                println!("  Date:   {}", chrono::DateTime::from_timestamp(ts, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "<unknown>".to_string()));
                println!();
                if let Some(msg) = commit.message() {
                    for line in msg.lines() {
                        println!("      {}", line);
                    }
                }
                println!();

                // Stat: show changed files count
                if stat {
                    if let Ok(parent) = commit.parent(0) {
                        let old_tree = parent.tree().ok();
                        let new_tree = commit.tree().ok();
                        if let (Some(old), Some(new)) = (old_tree, new_tree) {
                            let diff = self.repository().diff_tree_to_tree(Some(&old), Some(&new), None);
                            if let Ok(diff) = diff {
                                let stats = diff.stats()?;
                                println!("  {} files changed, {} insertions(+), {} deletions(-)",
                                    stats.files_changed(),
                                    stats.insertions(),
                                    stats.deletions()
                                );
                                println!();
                            }
                        }
                    }
                }
            }

            shown += 1;
        }

        Ok(())
    }

    /// Show reflog (HEAD movement history)
    pub fn show_reflog(&self, count: usize) -> Result<()> {
        let reflog = self.repo.reflog("HEAD")
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        println!("📋 Reflog (HEAD movements):");
        println!();

        for (i, entry) in reflog.iter().enumerate() {
            if i >= count {
                break;
            }
            let oid_short = entry.id_new().to_string();
            let oid_short = &oid_short[..7.min(oid_short.len())];
            let message = entry.message().unwrap_or("");
            println!("  {} {}", oid_short, message);
        }

        println!();
        println!("💡 Restore a state: torii save --reset <commit-hash> --reset-mode soft");

        Ok(())
    }

    /// Rebase with a pre-written todo file (no editor required)
    pub fn rebase_with_todo(&self, base: &str, todo_file: &std::path::Path) -> Result<()> {
        #[cfg(unix)]
        {
            let repo_path = self.repo.path().parent().unwrap().to_path_buf();
            let todo_abs = todo_file.canonicalize().map_err(|_| {
                crate::error::ToriiError::InvalidConfig(
                    format!("Todo file not found: {}", todo_file.display())
                )
            })?;
            println!("🔄 Rebasing from {} using todo file: {}", base, todo_abs.display());
            let editor = format!("cp {}", todo_abs.display());
            let status = std::process::Command::new("git")
                .args(["rebase", "-i", base])
                .env("GIT_SEQUENCE_EDITOR", &editor)
                .current_dir(&repo_path)
                .status()?;
            if !status.success() {
                eprintln!("⚠️  Rebase ended with conflicts or was aborted.");
            } else {
                println!("✅ Rebase complete");
            }
        }
        #[cfg(not(unix))]
        {
            let _ = (base, todo_file);
            return Err(crate::error::ToriiError::InvalidConfig(
                "Interactive rebase with todo file requires a Unix shell. Not supported on this platform.".to_string()
            ));
        }
        Ok(())
    }

    /// Interactive rebase
    pub fn rebase_interactive(&self, base: &str) -> Result<()> {
        #[cfg(unix)]
        {
            let repo_path = self.repo.path().parent().unwrap().to_path_buf();
            println!("🔄 Starting interactive rebase onto {}...", base);
            let status = std::process::Command::new("git")
                .args(["rebase", "-i", base])
                .current_dir(&repo_path)
                .status()?;
            if !status.success() {
                eprintln!("⚠️  Interactive rebase ended with conflicts or was aborted.");
            } else {
                println!("✅ Interactive rebase complete");
            }
        }
        #[cfg(not(unix))]
        {
            let _ = base;
            return Err(crate::error::ToriiError::InvalidConfig(
                "Interactive rebase requires a Unix terminal. Not supported on this platform.".to_string()
            ));
        }
        Ok(())
    }

    /// Continue an in-progress rebase
    pub fn rebase_continue(&self) -> Result<()> {
        // git2 doesn't expose a rebase-continue API; delegate to git when available
        let repo_path = self.repo.path().parent().unwrap().to_path_buf();
        if let Ok(status) = std::process::Command::new("git")
            .args(["rebase", "--continue"])
            .current_dir(&repo_path)
            .status()
        {
            if status.success() {
                println!("✅ Rebase continued");
            }
        } else {
            return Err(crate::error::ToriiError::InvalidConfig(
                "Could not continue rebase: git binary not found.".to_string()
            ));
        }
        Ok(())
    }

    /// Abort the current rebase
    pub fn rebase_abort(&self) -> Result<()> {
        let repo_path = self.repo.path().parent().unwrap().to_path_buf();
        if std::process::Command::new("git")
            .args(["rebase", "--abort"])
            .current_dir(&repo_path)
            .status()
            .is_ok()
        {
            println!("✅ Rebase aborted");
        } else {
            return Err(crate::error::ToriiError::InvalidConfig(
                "Could not abort rebase: git binary not found.".to_string()
            ));
        }
        Ok(())
    }

    /// Skip current patch in rebase
    pub fn rebase_skip(&self) -> Result<()> {
        let repo_path = self.repo.path().parent().unwrap().to_path_buf();
        if std::process::Command::new("git")
            .args(["rebase", "--skip"])
            .current_dir(&repo_path)
            .status()
            .is_ok()
        {
            println!("✅ Patch skipped");
        } else {
            return Err(crate::error::ToriiError::InvalidConfig(
                "Could not skip patch: git binary not found.".to_string()
            ));
        }
        Ok(())
    }

    /// Show changes
    pub fn diff(&self, staged: bool, last: bool) -> Result<()> {
        if last {
            // Show diff of last commit
            let head = self.repository().head()?.peel_to_commit()?;
            let tree = head.tree()?;
            
            let parent_tree = if head.parent_count() > 0 {
                Some(head.parent(0)?.tree()?)
            } else {
                None
            };
            
            let diff = self.repository().diff_tree_to_tree(
                parent_tree.as_ref(),
                Some(&tree),
                None,
            )?;
            
            self.print_diff(&diff)?;
        } else if staged {
            // Show staged changes
            let head = self.repository().head()?.peel_to_tree()?;
            let diff = self.repository().diff_tree_to_index(Some(&head), None, None)?;
            self.print_diff(&diff)?;
        } else {
            // Show unstaged changes
            let diff = self.repository().diff_index_to_workdir(None, None)?;
            self.print_diff(&diff)?;
        }
        
        Ok(())
    }

    fn print_diff(&self, diff: &git2::Diff) -> Result<()> {
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let origin = line.origin();
            let content = std::str::from_utf8(line.content()).unwrap_or("<binary>");
            
            match origin {
                '+' => print!("\x1b[32m+{}\x1b[0m", content),
                '-' => print!("\x1b[31m-{}\x1b[0m", content),
                _ => print!(" {}", content),
            }
            true
        })?;
        
        Ok(())
    }

    /// List local branches
    pub fn list_branches(&self) -> Result<Vec<String>> {
        let branches = self.repository().branches(Some(BranchType::Local))?;
        let mut branch_names = Vec::new();

        for branch in branches {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                branch_names.push(name.to_string());
            }
        }

        Ok(branch_names)
    }

    /// List remote branches
    pub fn list_remote_branches(&self) -> Result<Vec<String>> {
        let branches = self.repository().branches(Some(BranchType::Remote))?;
        let mut branch_names = Vec::new();

        for branch in branches {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                // Skip HEAD symrefs (e.g. origin/HEAD)
                if !name.ends_with("/HEAD") {
                    branch_names.push(name.to_string());
                }
            }
        }

        Ok(branch_names)
    }

    /// Create a new branch
    pub fn create_branch(&self, name: &str) -> Result<()> {
        let head = self.repository().head()?.peel_to_commit()?;
        self.repository().branch(name, &head, false)?;
        Ok(())
    }

    /// Delete a branch
    pub fn delete_branch(&self, name: &str) -> Result<()> {
        let mut branch = self.repository().find_branch(name, BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }

    /// Switch to a branch
    pub fn switch_branch(&self, name: &str) -> Result<()> {
        let obj = self.repository().revparse_single(&format!("refs/heads/{}", name))?;
        self.repository().checkout_tree(&obj, None)?;
        self.repository().set_head(&format!("refs/heads/{}", name))?;
        Ok(())
    }

    /// Clone a repository
    pub fn clone_repo(url: &str, directory: Option<&str>) -> Result<()> {
        let target = if let Some(dir) = directory {
            dir.to_string()
        } else {
            // Extract repo name from URL
            url.split('/')
                .last()
                .unwrap_or("repo")
                .trim_end_matches(".git")
                .to_string()
        };
        
        Repository::clone(url, &target)?;
        Ok(())
    }

    /// Rename a branch
    pub fn rename_branch(&self, old_name: &str, new_name: &str) -> Result<()> {
        let mut branch = self.repo.find_branch(old_name, git2::BranchType::Local)
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        branch.rename(new_name, false)
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        Ok(())
    }

    /// Rewrite commit history with new dates
    pub fn rewrite_history(&self, start_date: &str, end_date: &str) -> Result<()> {
        #[cfg(unix)]
        {
            println!("🔄 Rewriting commit history...");

            let start = NaiveDateTime::parse_from_str(&format!("{} +0200", start_date), "%Y-%m-%d %H:%M %z")
                .map_err(|e| crate::error::ToriiError::InvalidConfig(format!("Invalid start date: {}", e)))?;
            let end = NaiveDateTime::parse_from_str(&format!("{} +0200", end_date), "%Y-%m-%d %H:%M %z")
                .map_err(|e| crate::error::ToriiError::InvalidConfig(format!("Invalid end date: {}", e)))?;

            let mut revwalk = self.repo.revwalk()
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            revwalk.push_head().map_err(|e| crate::error::ToriiError::Git(e))?;
            revwalk.set_sorting(git2::Sort::REVERSE | git2::Sort::TIME)
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            let commits: Vec<String> = revwalk
                .filter_map(|r| r.ok())
                .map(|oid| oid.to_string())
                .collect();

            let total_commits = commits.len();
            if total_commits == 0 { return Ok(()); }

            let interval_seconds = (end.and_utc().timestamp() - start.and_utc().timestamp())
                / (total_commits as i64 - 1).max(1);

            let mut filter_script = String::new();
            for (i, commit_hash) in commits.iter().enumerate() {
                let new_timestamp = start.and_utc().timestamp() + (i as i64 * interval_seconds);
                let new_date = DateTime::from_timestamp(new_timestamp, 0)
                    .unwrap()
                    .format("%Y-%m-%d %H:%M:%S +0200");
                filter_script.push_str(&format!(
                    "if [ \"$GIT_COMMIT\" = \"{}\" ]; then\n    export GIT_AUTHOR_DATE=\"{}\"\n    export GIT_COMMITTER_DATE=\"{}\"\nfi\n",
                    commit_hash, new_date, new_date
                ));
            }

            let tmp = std::env::temp_dir().join("torii_filter.sh");
            std::fs::write(&tmp, &filter_script)?;

            let cmd = format!(
                "FILTER_BRANCH_SQUELCH_WARNING=1 git filter-branch -f --env-filter \"$(cat {})\" -- --all",
                tmp.display()
            );
            let output = std::process::Command::new("bash")
                .args(["-c", &cmd])
                .current_dir(self.repo.path().parent().unwrap())
                .output()?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(crate::error::ToriiError::InvalidConfig(
                    format!("Failed to rewrite history: {}", error)
                ));
            }

            println!("✅ Rewrote {} commits", total_commits);
        }
        #[cfg(not(unix))]
        {
            let _ = (start_date, end_date);
            return Err(crate::error::ToriiError::InvalidConfig(
                "History rewrite requires a Unix shell (bash + git filter-branch). Not supported on this platform.".to_string()
            ));
        }
        Ok(())
    }

    /// Remove a file from the entire git history
    pub fn remove_file_from_history(&self, file_path: &str) -> Result<()> {
        #[cfg(unix)]
        {
            let repo_path = self.repo.path().parent().unwrap();
            println!("🗑️  Removing '{}' from entire history...", file_path);
            let cmd = format!(
                "FILTER_BRANCH_SQUELCH_WARNING=1 git filter-branch -f --index-filter \
                'git rm -r --cached --ignore-unmatch {}' --tag-name-filter cat -- --all",
                file_path
            );
            let output = std::process::Command::new("bash")
                .args(["-c", &cmd])
                .current_dir(repo_path)
                .output()?;
            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(crate::error::ToriiError::InvalidConfig(
                    format!("Failed to remove file from history: {}", error)
                ));
            }
            println!("✅ '{}' removed from all commits", file_path);
            println!("💡 Run 'torii history clean' then 'torii sync --force' to update remote");
        }
        #[cfg(not(unix))]
        {
            let _ = file_path;
            return Err(crate::error::ToriiError::InvalidConfig(
                "Removing files from history requires a Unix shell. Not supported on this platform.".to_string()
            ));
        }
        Ok(())
    }

    /// Clean up repository (gc, reflog expire)
    pub fn clean_history(&self) -> Result<()> {
        println!("🧹 Cleaning repository...");

        // Remove filter-branch backup refs if they exist (cross-platform)
        let orig_refs = self.repo.path().join("refs").join("original");
        if orig_refs.exists() {
            let _ = std::fs::remove_dir_all(&orig_refs);
        }

        // git2 doesn't expose reflog expire or gc directly
        // Try git subprocess but don't fail if git binary is not present
        let repo_path = self.repo.path().parent().unwrap();
        let _ = std::process::Command::new("git")
            .args(["gc", "--prune=now", "--quiet"])
            .current_dir(repo_path)
            .output();

        println!("✅ Repository cleaned");
        Ok(())
    }

    /// Verify remote repository status
    pub fn verify_remote(&self) -> Result<()> {
        println!("🔍 Verifying remote status...\n");

        let local_oid = self.repo.head()
            .map_err(|e| crate::error::ToriiError::Git(e))?
            .target()
            .ok_or_else(|| crate::error::ToriiError::InvalidConfig("No HEAD".to_string()))?;

        let local_hash = local_oid.to_string();

        // Find remote tracking ref
        let branch = self.get_current_branch()?;
        let remote_ref = format!("refs/remotes/origin/{}", branch);
        let remote_hash = self.repo.find_reference(&remote_ref)
            .ok()
            .and_then(|r| r.target())
            .map(|oid| oid.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        println!("Local HEAD:  {}", &local_hash[..7.min(local_hash.len())]);
        println!("Remote HEAD: {}", &remote_hash[..7.min(remote_hash.len())]);

        if local_hash == remote_hash {
            println!("\n✅ Local and remote are in sync");
        } else {
            println!("\n⚠️  Local and remote have diverged");
            println!("💡 Use 'torii sync --force' to push local changes");
        }

        Ok(())
    }

    /// Fetch from remote without merging
    pub fn fetch(&self) -> Result<()> {
        println!("🔄 Fetching from remote...");

        let mut remote = self.repo.find_remote("origin")
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        let callbacks = GitRepo::ssh_callbacks();
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        remote.fetch(&[] as &[&str], Some(&mut fetch_options), None)
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        println!("✅ Fetched from remote");
        Ok(())
    }

    /// Revert a specific commit
    pub fn revert_commit(&self, commit_hash: &str) -> Result<()> {
        println!("🔄 Reverting commit {}...", commit_hash);

        let oid = git2::Oid::from_str(commit_hash)
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        let commit = self.repo.find_commit(oid)
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        self.repo.revert(&commit, None)
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        // Commit the revert
        let sig = self.repo.signature()
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        let mut index = self.repo.index()
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        let tree_oid = index.write_tree()
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        let tree = self.repo.find_tree(tree_oid)
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        let head = self.repo.head()
            .map_err(|e| crate::error::ToriiError::Git(e))?
            .peel_to_commit()
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        let msg = format!("Revert \"{}\"", commit.summary().unwrap_or(commit_hash));
        self.repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&head])
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        println!("✅ Reverted commit {}", &commit_hash[..7.min(commit_hash.len())]);
        Ok(())
    }

    /// Reset to a specific commit
    pub fn reset_commit(&self, commit_hash: &str, mode: &str) -> Result<()> {
        println!("🔄 Resetting to commit {} (mode: {})...", commit_hash, mode);

        let oid = git2::Oid::from_str(commit_hash)
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        let commit = self.repo.find_commit(oid)
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        let reset_type = match mode {
            "soft" => git2::ResetType::Soft,
            "hard" => git2::ResetType::Hard,
            _ => git2::ResetType::Mixed,
        };

        self.repo.reset(commit.as_object(), reset_type, None)
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        println!("✅ Reset to {}", &commit_hash[..7.min(commit_hash.len())]);
        Ok(())
    }

    /// Merge a branch into current branch
    pub fn merge_branch(&self, branch_name: &str) -> Result<()> {
        let branch_ref = format!("refs/heads/{}", branch_name);
        let annotated = self.repo.find_reference(&branch_ref)
            .map_err(|e| crate::error::ToriiError::Git(e))
            .and_then(|r| self.repo.reference_to_annotated_commit(&r)
                .map_err(|e| crate::error::ToriiError::Git(e)))?;

        let (analysis, _) = self.repo.merge_analysis(&[&annotated])
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        if analysis.is_up_to_date() {
            println!("Already up to date.");
            return Ok(());
        }

        if analysis.is_fast_forward() {
            let refname = format!("refs/heads/{}", self.get_current_branch()?);
            let mut reference = self.repo.find_reference(&refname)
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            reference.set_target(annotated.id(), "Fast-forward")
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            self.repo.set_head(&refname)
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            self.repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            println!("✅ Fast-forward merged {}", branch_name);
        } else {
            // Normal merge commit
            self.repo.merge(&[&annotated], None, None)
                .map_err(|e| crate::error::ToriiError::Git(e))?;

            let mut index = self.repo.index()
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            if index.has_conflicts() {
                println!("⚠️  Merge conflicts detected. Resolve them and run: torii save -m \"merge\"");
                return Ok(());
            }

            let tree_oid = index.write_tree()
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            let tree = self.repo.find_tree(tree_oid)
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            let sig = self.repo.signature()
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            let head = self.repo.head()
                .map_err(|e| crate::error::ToriiError::Git(e))?
                .peel_to_commit()
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            let branch_commit = self.repo.find_reference(&branch_ref)
                .map_err(|e| crate::error::ToriiError::Git(e))?
                .peel_to_commit()
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            let msg = format!("Merge branch '{}'", branch_name);
            self.repo.commit(Some("HEAD"), &sig, &sig, &msg, &tree, &[&head, &branch_commit])
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            self.repo.cleanup_state()
                .map_err(|e| crate::error::ToriiError::Git(e))?;

            println!("✅ Merged {}", branch_name);
        }

        Ok(())
    }

    /// Rebase current branch onto another branch
    pub fn rebase_branch(&self, branch_name: &str) -> Result<()> {
        // git2's Rebase API is available — use it for non-interactive rebase
        let branch_ref = format!("refs/heads/{}", branch_name);
        let upstream = self.repo.find_reference(&branch_ref)
            .map_err(|e| crate::error::ToriiError::Git(e))
            .and_then(|r| self.repo.reference_to_annotated_commit(&r)
                .map_err(|e| crate::error::ToriiError::Git(e)))?;

        let mut rebase = self.repo.rebase(None, Some(&upstream), None, None)
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        let sig = self.repo.signature()
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        while let Some(op) = rebase.next() {
            op.map_err(|e| crate::error::ToriiError::Git(e))?;
            let mut index = self.repo.index()
                .map_err(|e| crate::error::ToriiError::Git(e))?;
            if index.has_conflicts() {
                println!("⚠️  Rebase conflict. Resolve conflicts and run: torii history rebase --continue");
                return Ok(());
            }
            rebase.commit(None, &sig, None)
                .map_err(|e| crate::error::ToriiError::Git(e))?;
        }

        rebase.finish(Some(&sig))
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        println!("✅ Rebased onto {}", branch_name);
        Ok(())
    }

    /// List all tracked files in the index
    pub fn ls(&self, path_filter: Option<&str>) -> Result<()> {
        let mut index = self.repo.index()?;
        index.read(true)?;

        let entries: Vec<_> = index.iter()
            .filter(|e| {
                let path = String::from_utf8_lossy(&e.path).to_string();
                match path_filter {
                    Some(filter) => path.starts_with(filter),
                    None => true,
                }
            })
            .collect();

        if entries.is_empty() {
            println!("No tracked files.");
            return Ok(());
        }

        for entry in &entries {
            let path = String::from_utf8_lossy(&entry.path);
            println!("{}", path);
        }

        println!();
        println!("{} tracked file(s)", entries.len());

        Ok(())
    }

    /// Show details of a commit, tag, or file at a given ref
    pub fn show(&self, object: Option<&str>) -> Result<()> {
        let repo_path = self.repo.path().parent().unwrap();

        // Use the ref or default to HEAD
        let target = object.unwrap_or("HEAD");

        // Try to resolve as commit first
        let resolved = self.repo.revparse_single(target);

        match resolved {
            Ok(obj) => {
                match obj.kind() {
                    Some(git2::ObjectType::Commit) => {
                        let commit = obj.peel_to_commit()?;
                        let sig = commit.author();
                        let time = commit.time();
                        let timestamp = chrono::DateTime::from_timestamp(time.seconds(), 0)
                            .unwrap_or_default();

                        println!("commit {}", commit.id());
                        println!("Author: {} <{}>", sig.name().unwrap_or(""), sig.email().unwrap_or(""));
                        println!("Date:   {}", timestamp.format("%Y-%m-%d %H:%M:%S"));
                        println!();
                        println!("    {}", commit.message().unwrap_or("").trim());
                        println!();

                        // Show diff vs parent via git2
                        let commit_tree = commit.tree().ok();
                        let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
                        if let Some(new_tree) = commit_tree {
                            let diff = self.repo.diff_tree_to_tree(
                                parent_tree.as_ref(),
                                Some(&new_tree),
                                None,
                            )?;
                            diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                                let origin = line.origin();
                                let content = std::str::from_utf8(line.content()).unwrap_or("");
                                match origin {
                                    '+' => print!("\x1b[32m+{}\x1b[0m", content),
                                    '-' => print!("\x1b[31m-{}\x1b[0m", content),
                                    'H' | 'F' => print!("{}", content),
                                    _ => print!(" {}", content),
                                }
                                true
                            })?;
                        }
                    }
                    Some(git2::ObjectType::Tag) => {
                        let tag = obj.peel_to_tag()?;
                        println!("tag {}", tag.name().unwrap_or(""));
                        if let Some(tagger) = tag.tagger() {
                            println!("Tagger: {} <{}>", tagger.name().unwrap_or(""), tagger.email().unwrap_or(""));
                        }
                        println!();
                        println!("{}", tag.message().unwrap_or("").trim());
                    }
                    Some(git2::ObjectType::Blob) => {
                        let blob = obj.peel_to_blob()?;
                        let content = std::str::from_utf8(blob.content())
                            .unwrap_or("<binary>");
                        print!("{}", content);
                    }
                    _ => {
                        println!("{}", obj.id());
                    }
                }
            }
            Err(_) => {
                return Err(crate::error::ToriiError::InvalidConfig(
                    format!("Unknown ref or object: '{}'", target)
                ).into());
            }
        }

        Ok(())
    }
}
