// Extended Git operations for Torii
use git2::{Repository, BranchType, StatusOptions, DiffOptions};
use crate::error::Result;
use crate::core::GitRepo;
use std::process::Command;
use std::io::Write;
use chrono::{DateTime, NaiveDateTime};

impl GitRepo {
    /// Show commit history
    pub fn log(&self, count: Option<usize>, oneline: bool, _graph: bool) -> Result<()> {
        let mut revwalk = self.repository().revwalk()?;
        revwalk.push_head()?;
        
        let max_count = count.unwrap_or(10);
        let mut shown = 0;
        
        println!("📜 Commit History:");
        println!();
        
        for oid in revwalk {
            if shown >= max_count {
                break;
            }
            
            let oid = oid?;
            let commit = self.repository().find_commit(oid)?;
            
            if oneline {
                let short_id = &oid.to_string()[..7];
                let message = commit.message().unwrap_or("<no message>").lines().next().unwrap_or("");
                println!("  {} {}", short_id, message);
            } else {
                println!("  commit {}", oid);
                if let Some(author) = commit.author().name() {
                    println!("  Author: {}", author);
                }
                let time = commit.time();
                println!("  Date:   {}", chrono::DateTime::from_timestamp(time.seconds(), 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "<unknown>".to_string()));
                println!();
                if let Some(msg) = commit.message() {
                    for line in msg.lines() {
                        println!("      {}", line);
                    }
                }
                println!();
            }
            
            shown += 1;
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

    /// List branches
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
}
