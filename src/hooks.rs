use std::path::Path;
use std::process::Command;
use std::time::Instant;
use anyhow::{Result, anyhow};

use crate::toriignore::{HookRules, SizeRules, glob_match};

/// Execute every hook command in order. First non-zero exit aborts.
pub fn run_hooks(label: &str, commands: &[String], repo: &Path) -> Result<()> {
    if commands.is_empty() { return Ok(()); }
    println!("🪝 {} hooks: {} command(s)", label, commands.len());
    for cmd in commands {
        let start = Instant::now();
        print!("   → {} ", cmd);
        use std::io::Write;
        std::io::stdout().flush().ok();

        let status = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(repo)
            .status()
            .map_err(|e| anyhow!("failed to spawn `{}`: {}", cmd, e))?;

        let dur = start.elapsed();
        if !status.success() {
            let code = status.code().map(|c| c.to_string()).unwrap_or_else(|| "signal".into());
            return Err(anyhow!(
                "hook failed: `{}` exited with {} after {:.2}s — fix the issue or rerun with --skip-hooks",
                cmd, code, dur.as_secs_f64()
            ));
        }
        println!("✓ ({:.2}s)", dur.as_secs_f64());
    }
    Ok(())
}

/// Convenience: pre-save / pre-sync / post-* dispatch
pub fn pre_save(rules: &HookRules, repo: &Path) -> Result<()> {
    run_hooks("pre-save", &rules.pre_save, repo)
}
pub fn pre_sync(rules: &HookRules, repo: &Path) -> Result<()> {
    run_hooks("pre-sync", &rules.pre_sync, repo)
}
pub fn post_save(rules: &HookRules, repo: &Path) {
    let _ = run_hooks("post-save", &rules.post_save, repo);
}
pub fn post_sync(rules: &HookRules, repo: &Path) {
    let _ = run_hooks("post-sync", &rules.post_sync, repo);
}

/// Check staged file sizes against [size] limits.
/// Returns Err if any file exceeds `max`. Prints warnings for `warn` overruns.
pub fn check_size(rules: &SizeRules, repo: &Path, staged_paths: &[String]) -> Result<()> {
    if rules.max_bytes.is_none() && rules.warn_bytes.is_none() { return Ok(()); }

    let mut blocked: Vec<(String, u64)> = Vec::new();
    let mut warned: Vec<(String, u64)> = Vec::new();

    for rel in staged_paths {
        if rules.exclude.iter().any(|g| glob_match(rel, g)) { continue; }
        let abs = repo.join(rel);
        let size = match std::fs::metadata(&abs) {
            Ok(m) => m.len(),
            Err(_) => continue, // deleted file or unreadable
        };
        if let Some(max) = rules.max_bytes {
            if size > max { blocked.push((rel.clone(), size)); continue; }
        }
        if let Some(warn) = rules.warn_bytes {
            if size > warn { warned.push((rel.clone(), size)); }
        }
    }

    for (path, size) in &warned {
        println!("⚠️  large file: {} ({})", path, human_size(*size));
    }
    if !blocked.is_empty() {
        let mut msg = String::from("size limit exceeded:\n");
        for (path, size) in &blocked {
            msg.push_str(&format!("   {} — {}\n", path, human_size(*size)));
        }
        msg.push_str("\nAdjust [size] max in .toriignore, exclude these paths, or use git LFS.");
        return Err(anyhow!(msg));
    }
    Ok(())
}

fn human_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB { format!("{:.2} GB", bytes as f64 / GB as f64) }
    else if bytes >= MB { format!("{:.2} MB", bytes as f64 / MB as f64) }
    else if bytes >= KB { format!("{:.1} KB", bytes as f64 / KB as f64) }
    else { format!("{} B", bytes) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toriignore::SizeRules;

    #[test]
    fn human_size_boundaries() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(2048), "2.0 KB");
        assert_eq!(human_size(2 * 1024 * 1024), "2.00 MB");
    }

    #[test]
    fn size_check_blocks_oversize() {
        let dir = tempfile::tempdir().unwrap();
        let big = dir.path().join("big.bin");
        std::fs::write(&big, vec![0u8; 1024 * 1024]).unwrap(); // 1 MB
        let rules = SizeRules { max_bytes: Some(500 * 1024), warn_bytes: None, exclude: vec![] };
        let err = check_size(&rules, dir.path(), &["big.bin".to_string()]).unwrap_err();
        assert!(err.to_string().contains("size limit exceeded"));
    }

    #[test]
    fn size_check_respects_exclude() {
        let dir = tempfile::tempdir().unwrap();
        let big = dir.path().join("artwork.psd");
        std::fs::write(&big, vec![0u8; 1024 * 1024]).unwrap();
        let rules = SizeRules {
            max_bytes: Some(100),
            warn_bytes: None,
            exclude: vec!["*.psd".to_string()],
        };
        check_size(&rules, dir.path(), &["artwork.psd".to_string()]).unwrap();
    }

    #[test]
    fn size_check_skips_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let rules = SizeRules { max_bytes: Some(100), warn_bytes: None, exclude: vec![] };
        check_size(&rules, dir.path(), &["nonexistent".to_string()]).unwrap();
    }
}
