use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::{Result, ToriiError};
use crate::core::GitRepo;
// TODO: Implement ToriIgnore for snapshots
// use crate::toriignore::ToriIgnore;

#[derive(Debug, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub name: Option<String>,
    pub branch: String,
    pub commit_hash: Option<String>,
}

pub struct SnapshotManager {
    repo_path: PathBuf,
    snapshots_dir: PathBuf,
}

impl SnapshotManager {
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let repo_path = repo_path.as_ref().to_path_buf();
        let snapshots_dir = repo_path.join(".torii").join("snapshots");
        
        fs::create_dir_all(&snapshots_dir)?;

        Ok(Self {
            repo_path,
            snapshots_dir,
        })
    }

    /// Create a new snapshot
    pub fn create_snapshot(&self, name: Option<&str>) -> Result<String> {
        let repo = GitRepo::open(&self.repo_path)?;
        let timestamp = Utc::now();
        let id = timestamp.format("%Y%m%d_%H%M%S").to_string();
        
        let snapshot_dir = self.snapshots_dir.join(&id);
        fs::create_dir_all(&snapshot_dir)?;

        let branch = repo.get_current_branch()?;
        
        let metadata = SnapshotMetadata {
            id: id.clone(),
            timestamp,
            name: name.map(String::from),
            branch,
            commit_hash: None,
        };

        let metadata_path = snapshot_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)?;
        fs::write(metadata_path, metadata_json)?;

        self.create_bundle(&snapshot_dir, &repo)?;

        Ok(id)
    }

    /// Create a git bundle for the snapshot
    fn create_bundle(&self, snapshot_dir: &Path, repo: &GitRepo) -> Result<()> {
        // Create bundle with all refs
        let mut revwalk = repo.repository().revwalk()?;
        revwalk.push_head()?;
        
        // For now, we'll use a simpler approach: copy the entire .git directory
        // In production, we'd use proper git bundle creation
        let git_dir = self.repo_path.join(".git");
        let snapshot_git = snapshot_dir.join("git_backup");
        
        self.copy_dir_recursive(&git_dir, &snapshot_git)?;

        Ok(())
    }

    /// Recursively copy directory
    fn copy_dir_recursive(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;
        
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if file_type.is_dir() {
                self.copy_dir_recursive(&src_path, &dst_path)?;
            } else {
                fs::copy(&src_path, &dst_path)?;
            }
        }

        Ok(())
    }

    /// List all snapshots
    pub fn list_snapshots(&self) -> Result<()> {
        let entries = fs::read_dir(&self.snapshots_dir)?;
        
        println!("📸 Snapshots:");
        println!();

        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let metadata_path = entry.path().join("metadata.json");
                if metadata_path.exists() {
                    let metadata_json = fs::read_to_string(metadata_path)?;
                    let metadata: SnapshotMetadata = serde_json::from_str(&metadata_json)?;
                    
                    let name_str = metadata.name
                        .as_ref()
                        .map(|n| format!(" ({})", n))
                        .unwrap_or_default();
                    
                    println!("  {} - {}{}", 
                        metadata.id,
                        metadata.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        name_str
                    );
                    println!("    Branch: {}", metadata.branch);
                }
            }
        }

        Ok(())
    }

    /// Restore from a snapshot
    pub fn restore_snapshot(&self, id: &str) -> Result<()> {
        let snapshot_dir = self.snapshots_dir.join(id);
        
        if !snapshot_dir.exists() {
            return Err(ToriiError::Snapshot(format!("Snapshot not found: {}", id)));
        }

        let snapshot_git = snapshot_dir.join("git_backup");
        let git_dir = self.repo_path.join(".git");

        fs::remove_dir_all(&git_dir)?;
        self.copy_dir_recursive(&snapshot_git, &git_dir)?;

        // Reset working directory to match restored git state
        std::process::Command::new("git")
            .args(&["reset", "--hard", "HEAD"])
            .current_dir(&self.repo_path)
            .output()?;

        Ok(())
    }

    /// Delete a snapshot
    pub fn delete_snapshot(&self, id: &str) -> Result<()> {
        let snapshot_dir = self.snapshots_dir.join(id);
        
        if !snapshot_dir.exists() {
            return Err(ToriiError::Snapshot(format!("Snapshot not found: {}", id)));
        }

        fs::remove_dir_all(snapshot_dir)?;
        Ok(())
    }

    /// Configure auto-snapshot settings
    pub fn configure_auto_snapshot(&self, enable: bool, interval: Option<u32>) -> Result<()> {
        let config_path = self.repo_path.join(".torii").join("config.json");
        
        #[derive(Serialize, Deserialize)]
        struct Config {
            auto_snapshot_enabled: bool,
            auto_snapshot_interval_minutes: u32,
        }

        let config = Config {
            auto_snapshot_enabled: enable,
            auto_snapshot_interval_minutes: interval.unwrap_or(30),
        };

        let config_json = serde_json::to_string_pretty(&config)?;
        fs::write(config_path, config_json)?;

        Ok(())
    }

    /// Save work temporarily (like git stash)
    pub fn stash(&self, name: Option<&str>, _include_untracked: bool) -> Result<()> {
        let stash_name = name.unwrap_or("WIP");
        let snapshot_id = self.create_snapshot(Some(&format!("stash-{}", stash_name)))?;
        
        println!("📦 Stashed changes");
        println!("   ID: {}", snapshot_id);
        println!("   Name: {}", stash_name);
        println!();
        println!("💡 To restore: torii snapshot unstash");
        
        Ok(())
    }

    /// Restore stashed work
    pub fn unstash(&self, id: Option<&str>, keep: bool) -> Result<()> {
        let snapshot_id = if let Some(id) = id {
            id.to_string()
        } else {
            // Find latest stash
            let mut snapshots: Vec<_> = fs::read_dir(&self.snapshots_dir)?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    e.file_name().to_string_lossy().contains("stash-")
                })
                .collect();
            
            snapshots.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));
            
            let latest = snapshots.last()
                .ok_or_else(|| ToriiError::Snapshot("No stash found".to_string()))?;
            
            latest.file_name().to_string_lossy().to_string()
        };
        
        println!("🔄 Restoring stash: {}", snapshot_id);
        self.restore_snapshot(&snapshot_id)?;
        
        if !keep {
            self.delete_snapshot(&snapshot_id)?;
            println!("   Stash removed");
        }
        
        println!("✅ Stash restored");
        
        Ok(())
    }

    /// Undo last operation
    pub fn undo(&self) -> Result<()> {
        // Find most recent auto snapshot
        let mut snapshots: Vec<_> = fs::read_dir(&self.snapshots_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("before-") || name.contains("auto-")
            })
            .collect();
        
        snapshots.sort_by_key(|e| e.metadata().ok().and_then(|m| m.modified().ok()));
        
        let latest = snapshots.last()
            .ok_or_else(|| ToriiError::Snapshot("No operation to undo".to_string()))?;
        
        let snapshot_id = latest.file_name().to_string_lossy().to_string();
        
        println!("🔄 Undoing last operation...");
        println!("   Restoring snapshot: {}", snapshot_id);
        
        self.restore_snapshot(&snapshot_id)?;
        
        println!("✅ Operation undone");
        
        Ok(())
    }
}
