use crate::core::GitRepo;
use crate::error::Result;
use crate::snapshot::SnapshotManager;
use crate::integrate::IntegrateHelper;

impl GitRepo {
    /// Integrate changes from another branch (smart merge/rebase)
    pub fn integrate(&self, branch: &str, force_merge: bool, force_rebase: bool, preview: bool) -> Result<()> {
        let helper = IntegrateHelper::new(&self.repo);
        
        // Create safety snapshot
        let snapshot_mgr = SnapshotManager::new(".")?;
        let snapshot_id = snapshot_mgr.create_snapshot(Some(&format!("before-integrate-{}", branch)))?;
        println!("📸 Safety snapshot created: {}", snapshot_id);
        println!();
        
        // Get recommendation
        let recommendation = helper.get_recommendation(branch)?;
        let should_rebase = helper.should_rebase(branch)?;
        
        // Determine action
        let use_rebase = if force_merge {
            false
        } else if force_rebase {
            true
        } else {
            should_rebase
        };
        
        // Show analysis
        println!("🔍 Analyzing integration...\n");
        println!("{}", recommendation);
        println!();
        
        if preview {
            println!("💡 Preview mode - no changes made");
            println!();
            if use_rebase {
                println!("To proceed with rebase:");
                println!("  torii integrate {} --rebase", branch);
            } else {
                println!("To proceed with merge:");
                println!("  torii integrate {} --merge", branch);
            }
            return Ok(());
        }
        
        // Perform integration
        if use_rebase {
            println!("🔄 Rebasing onto {}...", branch);
            helper.rebase(branch)?;
            println!("✅ Rebase complete");
        } else {
            println!("🔄 Merging {}...", branch);
            helper.merge(branch)?;
            println!("✅ Merge complete");
        }
        
        println!();
        println!("💡 To undo: torii snapshot restore {}", snapshot_id);
        
        Ok(())
    }
}
