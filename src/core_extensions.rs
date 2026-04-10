// Extended Git operations for Torii
use git2::{Repository, BranchType, StatusOptions};
use crate::error::Result;
use crate::core::GitRepo;
use std::process::Command;

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
    pub fn list_branches(&self, all: bool) -> Result<()> {
        let branch_type = if all {
            None
        } else {
            Some(BranchType::Local)
        };
        
        let branches = self.repository().branches(branch_type)?;
        
        println!("🌿 Branches:");
        println!();
        
        for branch in branches {
            let (branch, branch_type) = branch?;
            let name = branch.name()?.unwrap_or("<unknown>");
            let is_head = branch.is_head();
            
            let marker = if is_head { "*" } else { " " };
            let type_str = match branch_type {
                BranchType::Local => "",
                BranchType::Remote => " (remote)",
            };
            
            if is_head {
                println!("  \x1b[32m{} {}\x1b[0m{}", marker, name, type_str);
            } else {
                println!("  {} {}{}", marker, name, type_str);
            }
        }
        
        Ok(())
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
}
