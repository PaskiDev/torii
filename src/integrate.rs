use git2::{Repository, BranchType, Oid};
use crate::error::Result;
use crate::snapshot::SnapshotManager;

pub struct IntegrateHelper<'repo> {
    repo: &'repo Repository,
}

impl<'repo> IntegrateHelper<'repo> {
    pub fn new(repo: &'repo Repository) -> Self {
        Self { repo }
    }

    /// Determine if we should merge or rebase based on context
    pub fn should_rebase(&self, target_branch: &str) -> Result<bool> {
        let head = self.repo.head()?;
        let current_branch = head.shorthand().unwrap_or("HEAD");
        
        // Check if current branch is main/master
        let is_main_branch = current_branch == "main" || current_branch == "master";
        
        // Check if target branch is main/master
        let is_target_main = target_branch == "main" || target_branch == "master";
        
        // Check if branch has been pushed to remote
        let has_remote = self.repo.find_branch(&format!("origin/{}", current_branch), BranchType::Remote).is_ok();
        
        // Decision logic:
        // - If on main/master → MERGE (preserve history)
        // - If integrating from main/master into feature → REBASE (clean history)
        // - If branch has remote → MERGE (safe, no rewrite)
        // - Otherwise → REBASE (clean history)
        
        if is_main_branch {
            Ok(false) // MERGE when on main
        } else if is_target_main {
            Ok(true) // REBASE when updating from main
        } else if has_remote {
            Ok(false) // MERGE if pushed to remote
        } else {
            Ok(true) // REBASE for local feature branches
        }
    }

    pub fn get_recommendation(&self, target_branch: &str) -> Result<String> {
        let should_rebase = self.should_rebase(target_branch)?;
        let head = self.repo.head()?;
        let current_branch = head.shorthand().unwrap_or("HEAD");
        
        if should_rebase {
            Ok(format!(
                "REBASE recommended\n\
                Reason: Updating feature branch '{}' with latest '{}'\n\
                This keeps history clean and linear",
                current_branch, target_branch
            ))
        } else {
            Ok(format!(
                "MERGE recommended\n\
                Reason: Integrating '{}' into '{}'\n\
                This preserves complete history",
                target_branch, current_branch
            ))
        }
    }

    pub fn merge(&self, target_branch: &str) -> Result<()> {
        let target_ref = self.repo.find_branch(target_branch, BranchType::Local)?;
        let target_commit = target_ref.get().peel_to_commit()?;
        
        let mut index = self.repo.index()?;
        let head_commit = self.repo.head()?.peel_to_commit()?;
        
        // Perform merge
        let merge_base = self.repo.merge_base(head_commit.id(), target_commit.id())?;
        let merge_base_tree = self.repo.find_commit(merge_base)?.tree()?;
        let our_tree = head_commit.tree()?;
        let their_tree = target_commit.tree()?;
        
        index.read_tree(&our_tree)?;
        
        let mut merge_options = git2::MergeOptions::new();
        let merge_result = self.repo.merge_trees(&merge_base_tree, &our_tree, &their_tree, Some(&mut merge_options))?;
        
        if merge_result.has_conflicts() {
            return Err(crate::error::ToriiError::Git(
                git2::Error::from_str("Merge conflicts detected. Please resolve manually.")
            ).into());
        }
        
        // Write merge result
        let tree_oid = self.repo.index()?.write_tree()?;
        let tree = self.repo.find_tree(tree_oid)?;
        
        // Create merge commit
        let sig = self.repo.signature()?;
        let msg = format!("Merge branch '{}'", target_branch);
        
        self.repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &msg,
            &tree,
            &[&head_commit, &target_commit],
        )?;
        
        Ok(())
    }

    pub fn rebase(&self, target_branch: &str) -> Result<()> {
        // For now, we'll use a simple implementation
        // A full rebase implementation would be more complex
        let target_ref = self.repo.find_branch(target_branch, BranchType::Local)?;
        let annotated_commit = self.repo.reference_to_annotated_commit(target_ref.get())?;
        
        // Simple rebase
        let mut rebase = self.repo.rebase(
            None,
            Some(&annotated_commit),
            None,
            None,
        )?;
        
        while let Some(op) = rebase.next() {
            let _op = op?;
            rebase.commit(None, &self.repo.signature()?, None)?;
        }
        
        rebase.finish(None)?;
        
        Ok(())
    }
}
