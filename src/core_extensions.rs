// Extended Git operations for Torii
use git2::{Repository, BranchType, StatusOptions, DiffOptions};
use crate::error::Result;
use crate::core::GitRepo;
use std::process::Command;
use std::io::Write;
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
        let repo_path = self.repo.path().parent().unwrap().to_path_buf();
        let output = Command::new("git")
            .args(["reflog", "--format=%gd %gs %H %ci", &format!("-{}", count)])
            .current_dir(&repo_path)
            .output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::ToriiError::InvalidConfig(
                format!("Failed to read reflog: {}", err)
            ));
        }

        println!("📋 Reflog (HEAD movements):");
        println!();

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            println!("  {}", line);
        }

        println!();
        println!("💡 Restore a state: torii save --reset <commit-hash> --reset-mode soft");

        Ok(())
    }

    /// Rebase with a pre-written todo file (no editor required)
    pub fn rebase_with_todo(&self, base: &str, todo_file: &std::path::Path) -> Result<()> {
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
            eprintln!("   Resolve conflicts then: torii rebase --continue");
            eprintln!("   Or abort with:          torii rebase --abort");
        } else {
            println!("✅ Rebase complete");
        }

        Ok(())
    }

    /// Interactive rebase
    pub fn rebase_interactive(&self, base: &str) -> Result<()> {
        let repo_path = self.repo.path().parent().unwrap().to_path_buf();
        println!("🔄 Starting interactive rebase onto {}...", base);

        let status = std::process::Command::new("git")
            .args(["rebase", "-i", base])
            .current_dir(&repo_path)
            .status()?;

        if !status.success() {
            eprintln!("⚠️  Interactive rebase ended with conflicts or was aborted.");
            eprintln!("   Resolve conflicts then: torii rebase --continue");
            eprintln!("   Or abort with:          torii rebase --abort");
        } else {
            println!("✅ Interactive rebase complete");
        }

        Ok(())
    }

    /// Continue an in-progress rebase
    pub fn rebase_continue(&self) -> Result<()> {
        let repo_path = self.repo.path().parent().unwrap().to_path_buf();
        let output = Command::new("git")
            .args(["rebase", "--continue"])
            .current_dir(&repo_path)
            .status()?;
        if output.success() {
            println!("✅ Rebase continued");
        }
        Ok(())
    }

    /// Abort the current rebase
    pub fn rebase_abort(&self) -> Result<()> {
        let repo_path = self.repo.path().parent().unwrap().to_path_buf();
        Command::new("git")
            .args(["rebase", "--abort"])
            .current_dir(&repo_path)
            .status()?;
        println!("✅ Rebase aborted");
        Ok(())
    }

    /// Skip current patch in rebase
    pub fn rebase_skip(&self) -> Result<()> {
        let repo_path = self.repo.path().parent().unwrap().to_path_buf();
        Command::new("git")
            .args(["rebase", "--skip"])
            .current_dir(&repo_path)
            .status()?;
        println!("✅ Patch skipped");
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
        // Use git command for renaming as git2 doesn't have a direct rename
        let output = Command::new("git")
            .args(&["branch", "-m", old_name, new_name])
            .current_dir(self.repo.path().parent().unwrap())
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::ToriiError::InvalidConfig(
                format!("Failed to rename branch: {}", error)
            ));
        }

        Ok(())
    }

    /// Rewrite commit history with new dates
    pub fn rewrite_history(&self, start_date: &str, end_date: &str) -> Result<()> {
        println!("🔄 Rewriting commit history...");
        
        // Parse dates
        let start = NaiveDateTime::parse_from_str(&format!("{} +0200", start_date), "%Y-%m-%d %H:%M %z")
            .map_err(|e| crate::error::ToriiError::InvalidConfig(format!("Invalid start date: {}", e)))?;
        let end = NaiveDateTime::parse_from_str(&format!("{} +0200", end_date), "%Y-%m-%d %H:%M %z")
            .map_err(|e| crate::error::ToriiError::InvalidConfig(format!("Invalid end date: {}", e)))?;

        // Get all commits
        let output = Command::new("git")
            .args(&["log", "--reverse", "--format=%H"])
            .current_dir(self.repo.path().parent().unwrap())
            .output()?;

        let commits: Vec<String> = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();

        let total_commits = commits.len();
        if total_commits == 0 {
            return Ok(());
        }

        // Create filter script
        let mut filter_script = String::new();
        let interval_seconds = (end.timestamp() - start.timestamp()) / (total_commits as i64 - 1).max(1);

        for (i, commit_hash) in commits.iter().enumerate() {
            let new_timestamp = start.timestamp() + (i as i64 * interval_seconds);
            let new_date = DateTime::from_timestamp(new_timestamp, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S +0200");
            
            filter_script.push_str(&format!(
                "if [ \"$GIT_COMMIT\" = \"{}\" ]; then\n    export GIT_AUTHOR_DATE=\"{}\"\n    export GIT_COMMITTER_DATE=\"{}\"\nfi\n",
                commit_hash, new_date, new_date
            ));
        }

        // Write filter script to temp file
        std::fs::write("/tmp/torii_filter.sh", &filter_script)?;

        // Run filter-branch
        let output = Command::new("bash")
            .args(&["-c", "FILTER_BRANCH_SQUELCH_WARNING=1 git filter-branch -f --env-filter \"$(cat /tmp/torii_filter.sh)\" -- --all"])
            .current_dir(self.repo.path().parent().unwrap())
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::ToriiError::InvalidConfig(
                format!("Failed to rewrite history: {}", error)
            ));
        }

        println!("✅ Rewrote {} commits", total_commits);
        Ok(())
    }

    /// Clean up repository (gc, reflog expire)
    pub fn clean_history(&self) -> Result<()> {
        println!("🧹 Cleaning repository...");
        
        let repo_path = self.repo.path().parent().unwrap();

        // Remove filter-branch refs
        let _ = Command::new("rm")
            .args(&["-rf", ".git/refs/original/"])
            .current_dir(repo_path)
            .output();

        // Expire reflog
        let output = Command::new("git")
            .args(&["reflog", "expire", "--expire=now", "--all"])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::ToriiError::InvalidConfig(
                format!("Failed to expire reflog: {}", error)
            ));
        }

        // Run gc
        let output = Command::new("git")
            .args(&["gc", "--prune=now"])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::ToriiError::InvalidConfig(
                format!("Failed to run gc: {}", error)
            ));
        }

        Ok(())
    }

    /// Verify remote repository status
    pub fn verify_remote(&self) -> Result<()> {
        println!("🔍 Verifying remote status...\n");
        
        let repo_path = self.repo.path().parent().unwrap();

        // Get local HEAD
        let local_output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(repo_path)
            .output()?;

        let local_hash = String::from_utf8_lossy(&local_output.stdout).trim().to_string();

        // Get remote HEAD
        let remote_output = Command::new("git")
            .args(&["ls-remote", "origin", "main"])
            .current_dir(repo_path)
            .output()?;

        if !remote_output.status.success() {
            println!("❌ Failed to connect to remote");
            return Ok(());
        }

        let remote_line = String::from_utf8_lossy(&remote_output.stdout);
        let remote_hash = remote_line.split_whitespace().next().unwrap_or("");

        println!("Local HEAD:  {}", &local_hash[..7.min(local_hash.len())]);
        println!("Remote HEAD: {}", &remote_hash[..7.min(remote_hash.len())]);

        if local_hash.starts_with(remote_hash) || remote_hash.starts_with(&local_hash) {
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
        
        let repo_path = self.repo.path().parent().unwrap();

        let output = Command::new("git")
            .args(&["fetch", "origin"])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::ToriiError::InvalidConfig(
                format!("Failed to fetch: {}", error)
            ));
        }

        Ok(())
    }

    /// Revert a specific commit
    pub fn revert_commit(&self, commit_hash: &str) -> Result<()> {
        println!("🔄 Reverting commit {}...", commit_hash);
        
        let repo_path = self.repo.path().parent().unwrap();

        let output = Command::new("git")
            .args(&["revert", "--no-edit", commit_hash])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::ToriiError::InvalidConfig(
                format!("Failed to revert commit: {}", error)
            ));
        }

        Ok(())
    }

    /// Reset to a specific commit
    pub fn reset_commit(&self, commit_hash: &str, mode: &str) -> Result<()> {
        println!("🔄 Resetting to commit {} (mode: {})...", commit_hash, mode);
        
        let repo_path = self.repo.path().parent().unwrap();

        let reset_flag = match mode {
            "soft" => "--soft",
            "hard" => "--hard",
            _ => "--mixed", // default
        };

        let output = Command::new("git")
            .args(&["reset", reset_flag, commit_hash])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::ToriiError::InvalidConfig(
                format!("Failed to reset: {}", error)
            ));
        }

        Ok(())
    }

    /// Merge a branch into current branch
    pub fn merge_branch(&self, branch_name: &str) -> Result<()> {
        let repo_path = self.repo.path().parent().unwrap();

        let output = Command::new("git")
            .args(&["merge", branch_name])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::ToriiError::InvalidConfig(
                format!("Failed to merge branch: {}", error)
            ));
        }

        Ok(())
    }

    /// Rebase current branch onto another branch
    pub fn rebase_branch(&self, branch_name: &str) -> Result<()> {
        let repo_path = self.repo.path().parent().unwrap();

        let output = Command::new("git")
            .args(&["rebase", branch_name])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::ToriiError::InvalidConfig(
                format!("Failed to rebase: {}", error)
            ));
        }

        Ok(())
    }
}
