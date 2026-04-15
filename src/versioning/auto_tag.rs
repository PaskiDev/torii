use anyhow::Result;
use crate::versioning::conventional::{ConventionalCommit, CommitType};
use crate::versioning::semver::{Version, VersionBump};
use crate::core::GitRepo;

pub struct AutoTagger {
    repo: GitRepo,
    prefix: String,
}

impl AutoTagger {
    pub fn new(repo: GitRepo) -> Self {
        Self {
            repo,
            prefix: "v".to_string(),
        }
    }
    
    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.prefix = prefix;
        self
    }
    
    /// Get the latest version tag from the repository
    pub fn get_latest_version(&self) -> Result<Option<Version>> {
        let tags = self.repo.get_tags_list()?;
        
        let mut versions: Vec<Version> = tags
            .iter()
            .filter_map(|tag| {
                tag.strip_prefix(&self.prefix)
                    .and_then(|v| v.parse::<Version>().ok())
            })
            .collect();
        
        versions.sort();
        Ok(versions.last().copied())
    }
    
    /// Determine the version bump type based on commit message
    pub fn determine_bump(&self, commit_msg: &str) -> Result<VersionBump> {
        let commit = ConventionalCommit::parse(commit_msg)?;
        
        if commit.is_breaking() {
            return Ok(VersionBump::Major);
        }
        
        match commit.commit_type {
            CommitType::Feat => Ok(VersionBump::Minor),
            CommitType::Fix | CommitType::Perf => Ok(VersionBump::Patch),
            _ => Ok(VersionBump::None),
        }
    }
    
    /// Calculate the next version based on commit message
    pub fn calculate_next_version(&self, commit_msg: &str) -> Result<Option<Version>> {
        let bump = self.determine_bump(commit_msg)?;
        
        if bump == VersionBump::None {
            return Ok(None);
        }
        
        let current_version = self.get_latest_version()?
            .unwrap_or_else(Version::initial);
        
        Ok(Some(current_version.bump(bump)))
    }
    
    /// Create a tag for the given version
    pub fn create_tag(&self, version: &Version, message: &str) -> Result<()> {
        let tag_name = format!("{}{}", self.prefix, version);
        self.repo.create_tag(&tag_name, Some(message))?;
        println!("✅ Created tag: {}", tag_name);
        Ok(())
    }
    
    /// Auto-tag based on commit message
    pub fn auto_tag(&self, commit_msg: &str) -> Result<()> {
        if let Some(version) = self.calculate_next_version(commit_msg)? {
            self.create_tag(&version, commit_msg)?;
        } else {
            println!("ℹ️  No version tag created (commit type doesn't trigger versioning)");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_determine_bump_feat() {
        let repo = GitRepo::open(".").unwrap();
        let tagger = AutoTagger::new(repo);
        let bump = tagger.determine_bump("feat: add new feature").unwrap();
        assert_eq!(bump, VersionBump::Minor);
    }
    
    #[test]
    fn test_determine_bump_fix() {
        let repo = GitRepo::open(".").unwrap();
        let tagger = AutoTagger::new(repo);
        let bump = tagger.determine_bump("fix: resolve bug").unwrap();
        assert_eq!(bump, VersionBump::Patch);
    }
    
    #[test]
    fn test_determine_bump_breaking() {
        let repo = GitRepo::open(".").unwrap();
        let tagger = AutoTagger::new(repo);
        let bump = tagger.determine_bump("feat!: breaking change").unwrap();
        assert_eq!(bump, VersionBump::Major);
    }
    
    #[test]
    fn test_determine_bump_docs() {
        let repo = GitRepo::open(".").unwrap();
        let tagger = AutoTagger::new(repo);
        let bump = tagger.determine_bump("docs: update README").unwrap();
        assert_eq!(bump, VersionBump::None);
    }
}
