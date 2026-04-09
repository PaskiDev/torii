use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, BufRead};
use anyhow::Result;

/// Manages .toriignore patterns
pub struct ToriIgnore {
    patterns: Vec<String>,
}

impl ToriIgnore {
    /// Load .toriignore from repository root, fallback to .gitignore if not found
    pub fn load<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let repo_path = repo_path.as_ref();
        
        // Try .toriignore first
        let toriignore_path = repo_path.join(".toriignore");
        if toriignore_path.exists() {
            return Self::from_file(&toriignore_path);
        }
        
        // Fallback to .gitignore for compatibility
        let gitignore_path = repo_path.join(".gitignore");
        if gitignore_path.exists() {
            return Self::from_file(&gitignore_path);
        }
        
        // No ignore file found, return empty
        Ok(Self {
            patterns: Vec::new(),
        })
    }
    
    /// Load patterns from a file
    fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        
        let mut patterns = Vec::new();
        
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            patterns.push(line.to_string());
        }
        
        Ok(Self { patterns })
    }
    
    /// Check if a path should be ignored
    pub fn is_ignored<P: AsRef<Path>>(&self, path: P) -> bool {
        let path_str = path.as_ref().to_string_lossy();
        
        for pattern in &self.patterns {
            if self.matches_pattern(&path_str, pattern) {
                return true;
            }
        }
        
        false
    }
    
    /// Simple pattern matching (supports *, /, and **)
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        // Exact match
        if path == pattern {
            return true;
        }
        
        // Directory match (pattern ends with /)
        if pattern.ends_with('/') {
            let dir_pattern = pattern.trim_end_matches('/');
            if path.starts_with(dir_pattern) {
                return true;
            }
        }
        
        // Wildcard match
        if pattern.contains('*') {
            return self.wildcard_match(path, pattern);
        }
        
        // Extension match (*.ext)
        if pattern.starts_with("*.") {
            let ext = pattern.trim_start_matches("*.");
            if path.ends_with(&format!(".{}", ext)) {
                return true;
            }
        }
        
        // Path contains pattern
        if path.contains(pattern) {
            return true;
        }
        
        false
    }
    
    /// Simple wildcard matching
    fn wildcard_match(&self, path: &str, pattern: &str) -> bool {
        // Handle ** (match any directory depth)
        if pattern.contains("**/") {
            let parts: Vec<&str> = pattern.split("**/").collect();
            if parts.len() == 2 {
                let suffix = parts[1];
                if path.contains(suffix) || path.ends_with(suffix) {
                    return true;
                }
            }
        }
        
        // Handle simple * wildcard
        if pattern.starts_with('*') && pattern.ends_with('*') {
            let middle = pattern.trim_matches('*');
            return path.contains(middle);
        }
        
        if pattern.starts_with('*') {
            let suffix = pattern.trim_start_matches('*');
            return path.ends_with(suffix);
        }
        
        if pattern.ends_with('*') {
            let prefix = pattern.trim_end_matches('*');
            return path.starts_with(prefix);
        }
        
        false
    }
    
    /// Get all patterns
    pub fn patterns(&self) -> &[String] {
        &self.patterns
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_exact_match() {
        let ignore = ToriIgnore {
            patterns: vec!["target".to_string()],
        };
        
        assert!(ignore.is_ignored("target"));
        assert!(!ignore.is_ignored("src"));
    }
    
    #[test]
    fn test_directory_match() {
        let ignore = ToriIgnore {
            patterns: vec!["node_modules/".to_string()],
        };
        
        assert!(ignore.is_ignored("node_modules/package.json"));
        assert!(!ignore.is_ignored("src/main.rs"));
    }
    
    #[test]
    fn test_extension_match() {
        let ignore = ToriIgnore {
            patterns: vec!["*.log".to_string()],
        };
        
        assert!(ignore.is_ignored("debug.log"));
        assert!(ignore.is_ignored("error.log"));
        assert!(!ignore.is_ignored("README.md"));
    }
    
    #[test]
    fn test_wildcard_match() {
        let ignore = ToriIgnore {
            patterns: vec!["**/temp/*".to_string()],
        };
        
        assert!(ignore.is_ignored("src/temp/file.txt"));
        assert!(ignore.is_ignored("temp/data"));
    }
}
