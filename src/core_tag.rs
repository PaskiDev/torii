use crate::core::GitRepo;
use crate::tag::TagManager;
use std::process::Command;
use anyhow::Result;

impl GitRepo {
    /// Create a tag
    pub fn create_tag(&self, name: &str, message: Option<&str>) -> Result<()> {
        let tag_mgr = TagManager::new(&self.repo);
        tag_mgr.create_tag(name, message)?;
        Ok(())
    }

    /// List all tags
    pub fn list_tags(&self) -> Result<()> {
        let tag_mgr = TagManager::new(&self.repo);
        let tags = tag_mgr.list_tags()?;
        
        if tags.is_empty() {
            println!("No tags found");
        } else {
            println!("📌 Tags:");
            for tag in tags {
                println!("  {}", tag);
            }
        }
        
        Ok(())
    }

    /// Get tags as a vector (for internal use)
    pub fn get_tags_list(&self) -> Result<Vec<String>> {
        let tag_mgr = TagManager::new(&self.repo);
        Ok(tag_mgr.list_tags()?)
    }

    /// Delete a tag
    pub fn delete_tag(&self, name: &str) -> Result<()> {
        let tag_mgr = TagManager::new(&self.repo);
        tag_mgr.delete_tag(name)?;
        Ok(())
    }

    /// Push tags to remote
    pub fn push_tags(&self, name: Option<&str>) -> Result<()> {
        let output = if let Some(tag) = name {
            Command::new("git")
                .args(&["push", "origin", tag])
                .current_dir(self.repo.path().parent().unwrap())
                .output()?
        } else {
            Command::new("git")
                .args(&["push", "origin", "--tags"])
                .current_dir(self.repo.path().parent().unwrap())
                .output()?
        };

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::Error::msg(format!("Failed to push tags: {}", error)));
        }

        Ok(())
    }

    /// Show tag details
    pub fn show_tag(&self, name: &str) -> Result<()> {
        let tag_mgr = TagManager::new(&self.repo);
        tag_mgr.show_tag(name)?;
        Ok(())
    }

    /// Cherry-pick a commit
    pub fn cherry_pick(&self, commit_hash: &str) -> Result<()> {
        let oid = git2::Oid::from_str(commit_hash)?;
        let commit = self.repo.find_commit(oid)?;
        
        println!("🍒 Cherry-picking commit: {}", commit.id());
        
        // Perform cherry-pick
        self.repo.cherrypick(&commit, None)?;
        
        // Check for conflicts
        let mut index = self.repo.index()?;
        if index.has_conflicts() {
            println!("⚠️  Conflicts detected!");
            println!("💡 Resolve conflicts and run: torii cherry-pick --continue");
            return Ok(());
        }
        
        // Create commit
        let sig = self.repo.signature()?;
        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;
        let head = self.repo.head()?.peel_to_commit()?;
        
        self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            commit.message().unwrap_or("Cherry-picked commit"),
            &tree,
            &[&head],
        )?;
        
        println!("✅ Cherry-pick complete");
        
        Ok(())
    }

    /// Continue cherry-pick after resolving conflicts
    pub fn cherry_pick_continue(&self) -> Result<()> {
        println!("🔄 Continuing cherry-pick...");
        
        let sig = self.repo.signature()?;
        let mut index = self.repo.index()?;
        let tree_oid = index.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;
        let head = self.repo.head()?.peel_to_commit()?;
        
        self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Cherry-picked commit",
            &tree,
            &[&head],
        )?;
        
        println!("✅ Cherry-pick complete");
        
        Ok(())
    }

    /// Abort cherry-pick
    pub fn cherry_pick_abort(&self) -> Result<()> {
        println!("❌ Aborting cherry-pick...");
        
        // Reset to HEAD
        let head = self.repo.head()?.peel_to_commit()?;
        self.repo.reset(head.as_object(), git2::ResetType::Hard, None)?;
        
        println!("✅ Cherry-pick aborted");
        
        Ok(())
    }

    /// Show blame for a file
    pub fn blame(&self, file: &str, lines: Option<&str>) -> Result<()> {
        let blame = self.repo.blame_file(std::path::Path::new(file), None)?;
        
        let (start_line, end_line) = if let Some(range) = lines {
            let parts: Vec<&str> = range.split(',').collect();
            if parts.len() == 2 {
                let start = parts[0].parse::<usize>().unwrap_or(1);
                let end = parts[1].parse::<usize>().unwrap_or(usize::MAX);
                (start, end)
            } else {
                (1, usize::MAX)
            }
        } else {
            (1, usize::MAX)
        };
        
        println!("📝 Blame for: {}", file);
        println!();
        
        for (idx, hunk) in blame.iter().enumerate() {
            let line_num = idx + 1;
            
            if line_num < start_line || line_num > end_line {
                continue;
            }
            
            let commit_id = hunk.final_commit_id();
            let commit = self.repo.find_commit(commit_id)?;
            let author = commit.author();
            let short_id = format!("{:.7}", commit_id);
            
            println!(
                "{} {} ({}) {}",
                short_id,
                author.name().unwrap_or("Unknown"),
                line_num,
                commit.message().unwrap_or("").lines().next().unwrap_or("")
            );
        }
        
        Ok(())
    }
}
