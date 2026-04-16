use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::{Result, ToriiError};
use crate::core::GitRepo;

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

        // Reset working directory to match restored git state via git2
        {
            let repo = git2::Repository::discover(&self.repo_path)
                .map_err(|e| ToriiError::Git(e))?;
            let head = repo.head()
                .map_err(|e| ToriiError::Git(e))?
                .peel_to_commit()
                .map_err(|e| ToriiError::Git(e))?;
            repo.reset(
                head.as_object(),
                git2::ResetType::Hard,
                Some(git2::build::CheckoutBuilder::default().force()),
            ).map_err(|e| ToriiError::Git(e))?;
        }

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
    pub fn stash(&self, name: Option<&str>, include_untracked: bool) -> Result<()> {
        let stash_name = name.unwrap_or("WIP");

        // Stage untracked files via git2 intent-to-add so snapshot captures them
        if include_untracked {
            if let Ok(repo) = git2::Repository::discover(&self.repo_path) {
                if let Ok(mut index) = repo.index() {
                    let _ = index.add_all(
                        ["*"].iter(),
                        git2::IndexAddOption::DEFAULT | git2::IndexAddOption::CHECK_PATHSPEC,
                        None,
                    );
                    let _ = index.write();
                }
            }
        }

        let snapshot_id = self.create_snapshot(Some(&format!("stash-{}", stash_name)))?;

        // Reset to HEAD via git2, discarding all changes
        {
            let repo = git2::Repository::discover(&self.repo_path)
                .map_err(|e| ToriiError::Git(e))?;
            let head = repo.head()
                .map_err(|e| ToriiError::Git(e))?
                .peel_to_commit()
                .map_err(|e| ToriiError::Git(e))?;
            repo.reset(
                head.as_object(),
                git2::ResetType::Hard,
                Some(git2::build::CheckoutBuilder::default().force()),
            ).map_err(|e| ToriiError::Git(e))?;

            // Remove untracked files if requested
            if include_untracked {
                let mut opts = git2::StatusOptions::new();
                opts.include_untracked(true).recurse_untracked_dirs(true);
                if let Ok(statuses) = repo.statuses(Some(&mut opts)) {
                    for entry in statuses.iter() {
                        if entry.status().is_wt_new() {
                            if let Some(path) = entry.path() {
                                let full_path = self.repo_path.join(path);
                                if full_path.is_dir() {
                                    let _ = std::fs::remove_dir_all(&full_path);
                                } else {
                                    let _ = std::fs::remove_file(&full_path);
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("📦 Stashed changes");
        println!("   ID: {}", snapshot_id);
        println!("   Name: {}", stash_name);
        if include_untracked {
            println!("   Untracked files included");
        }
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
