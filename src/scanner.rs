// Sensitive data scanner — runs before every commit
use std::path::Path;
use crate::error::Result;

/// A detected sensitive pattern in a file
pub struct Finding {
    pub file: String,
    pub line: usize,
    pub pattern_name: String,
    pub preview: String,
}

/// Patterns that indicate sensitive data
struct Pattern {
    name: &'static str,
    /// Returns true if the line matches
    detect: fn(&str) -> bool,
}

fn mask(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= 8 {
        return "*".repeat(chars.len());
    }
    let visible = 4;
    format!("{}{}",
        &chars[..visible].iter().collect::<String>(),
        "*".repeat(chars.len() - visible)
    )
}

const PATTERNS: &[Pattern] = &[
    Pattern {
        name: "Private key (PEM)",
        detect: |l| l.contains("-----BEGIN") && (
            l.contains("PRIVATE KEY") ||
            l.contains("RSA PRIVATE") ||
            l.contains("EC PRIVATE")
        ),
    },
    Pattern {
        name: "JWT token",
        detect: |l| {
            // eyJ... base64 header — at least 3 segments
            l.split_whitespace().any(|w| {
                let w = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '_' && c != '-');
                let parts: Vec<&str> = w.split('.').collect();
                parts.len() == 3
                    && parts[0].starts_with("eyJ")
                    && parts[0].len() > 10
                    && parts[1].len() > 10
            })
        },
    },
    Pattern {
        name: "AWS access key",
        detect: |l| {
            l.split_whitespace().any(|w| {
                let w = w.trim_matches(|c: char| !c.is_alphanumeric());
                (w.starts_with("AKIA") || w.starts_with("ASIA") || w.starts_with("AROA"))
                    && w.len() == 20
                    && w.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            })
        },
    },
    Pattern {
        name: "AWS secret key",
        detect: |l| {
            let lower = l.to_lowercase();
            (lower.contains("aws_secret") || lower.contains("aws secret"))
                && (l.contains('=') || l.contains(':'))
                && l.len() > 40
        },
    },
    Pattern {
        name: "GitHub/GitLab token",
        detect: |l| {
            l.split_whitespace().any(|w| {
                let w = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
                w.starts_with("ghp_") || w.starts_with("gho_") ||
                w.starts_with("ghs_") || w.starts_with("github_pat_") ||
                w.starts_with("glpat-") || w.starts_with("glptt-")
            })
        },
    },
    Pattern {
        name: "Generic API key / token",
        detect: |l| {
            let lower = l.to_lowercase();
            let has_key_word =
                lower.contains("api_key") || lower.contains("api_secret") ||
                lower.contains("auth_token") || lower.contains("access_token") ||
                lower.contains("secret_key") || lower.contains("private_key") ||
                lower.contains("password") || lower.contains("passwd") ||
                lower.contains("auth_token");
            let has_assignment = l.contains('=') || l.contains(':');
            let has_value = l.split(&['=', ':'][..])
                .nth(1)
                .map(|v| {
                    let v = v.trim().trim_matches(|c: char| c == '"' || c == '\'' || c == '`');
                    let vl = v.to_lowercase();
                    // Real secrets: no spaces, no sentence punctuation, min length
                    let looks_like_secret = v.len() >= 16
                        && !v.contains(' ')
                        && !v.contains('.')  // sentences have dots
                        && !v.starts_with("${")
                        && !v.starts_with("$(")
                        && !v.starts_with("process.env")
                        && !v.starts_with("env.")
                        && !v.starts_with("os.environ")
                        && !v.starts_with("<")
                        // English placeholders
                        && !vl.eq("your_secret_here")
                        && !vl.eq("changeme")
                        && !vl.eq("placeholder")
                        && !vl.eq("todo")
                        && !vl.starts_with("your_")
                        && !vl.starts_with("my_")
                        && !vl.contains("example")
                        && !vl.contains("sample")
                        && !vl.contains("replace")
                        && !vl.contains("change_me")
                        && !vl.contains("insert")
                        // Spanish placeholders
                        && !vl.starts_with("tu_")
                        && !vl.starts_with("mi_")
                        && !vl.contains("cambiar")
                        && !vl.contains("reemplazar")
                        && !vl.contains("ejemplo")
                        && !vl.contains("aqui")
                        && !vl.contains("pon_")
                        && !vl.contains("escribe");
                    looks_like_secret
                })
                .unwrap_or(false);
            has_key_word && has_assignment && has_value
        },
    },
    Pattern {
        name: "Database connection string with credentials",
        detect: |l| {
            let lower = l.to_lowercase();
            (lower.contains("postgresql://") || lower.contains("mysql://") ||
             lower.contains("mongodb://") || lower.contains("redis://") ||
             lower.contains("libsql://") || lower.contains("turso://"))
                && l.contains('@')
                && !l.contains("user:password@")
                && !l.contains("user:pass@")
                && !l.contains("<password>")
        },
    },
    Pattern {
        name: "Stripe key",
        detect: |l| {
            l.split_whitespace().any(|w| {
                let w = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                w.starts_with("sk_live_") || w.starts_with("pk_live_") ||
                w.starts_with("rk_live_")
            })
        },
    },
    Pattern {
        name: "Twilio / SendGrid / Brevo key",
        detect: |l| {
            l.split_whitespace().any(|w| {
                let w = w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                // SG. prefix = SendGrid
                (w.starts_with("SG.") && w.len() > 40) ||
                // AC... = Twilio account SID
                (w.starts_with("AC") && w.len() == 34 && w.chars().all(|c| c.is_ascii_alphanumeric()))
            })
        },
    },
];

/// Extensions/suffixes that are safe to commit with example values
fn is_example_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".example")
        || lower.ends_with(".sample")
        || lower.ends_with(".template")
        || lower.ends_with(".example.env")
        || lower.ends_with(".env.example")
        || lower.ends_with(".env.sample")
        || lower.ends_with(".env.template")
        || lower.contains(".example.")
        || lower.contains(".sample.")
}

/// Binary-like or generated files to skip
fn should_skip_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".lock")
        || lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".gif")
        || lower.ends_with(".svg")
        || lower.ends_with(".ico")
        || lower.ends_with(".wasm")
        || lower.ends_with(".pdf")
        || lower.ends_with(".zip")
        || lower.contains("bun.lock")
        || lower.contains("package-lock")
        || lower.contains("yarn.lock")
}

/// Scan staged files in the git index for sensitive data.
/// Returns a list of findings.
pub fn scan_staged(repo_path: &Path) -> Result<Vec<Finding>> {
    use std::process::Command;

    let mut findings = Vec::new();

    // Get list of staged files
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(repo_path)
        .output()?;

    let staged_files = String::from_utf8_lossy(&output.stdout);

    for file_path in staged_files.lines() {
        if is_example_file(file_path) || should_skip_file(file_path) {
            continue;
        }

        // Get staged content of the file
        let content = Command::new("git")
            .args(["show", &format!(":{}", file_path)])
            .current_dir(repo_path)
            .output();

        let content = match content {
            Ok(c) if c.status.success() => String::from_utf8_lossy(&c.stdout).to_string(),
            _ => continue,
        };

        for (line_num, line) in content.lines().enumerate() {
            // Skip comments
            let trimmed = line.trim();
            if trimmed.starts_with('#') || trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            for pattern in PATTERNS {
                if (pattern.detect)(line) {
                    let preview = mask(line.trim());
                    findings.push(Finding {
                        file: file_path.to_string(),
                        line: line_num + 1,
                        pattern_name: pattern.name.to_string(),
                        preview,
                    });
                    break; // one finding per line is enough
                }
            }
        }
    }

    Ok(findings)
}

/// Scan an entire git history for sensitive data (for migration use).
/// Returns findings grouped by commit.
pub fn scan_history(repo_path: &Path) -> Result<Vec<(String, Vec<Finding>)>> {
    use std::process::Command;

    let mut results = Vec::new();

    // Get all commit hashes
    let log = Command::new("git")
        .args(["log", "--format=%H %s", "--all"])
        .current_dir(repo_path)
        .output()?;

    let commits: Vec<(String, String)> = String::from_utf8_lossy(&log.stdout)
        .lines()
        .filter_map(|l| {
            let mut parts = l.splitn(2, ' ');
            let hash = parts.next()?.to_string();
            let msg = parts.next().unwrap_or("").to_string();
            Some((hash, msg))
        })
        .collect();

    println!("🔍 Scanning {} commits...", commits.len());

    for (hash, subject) in &commits {
        // Get files changed in this commit
        let files = Command::new("git")
            .args(["diff-tree", "--no-commit-id", "-r", "--name-only", hash])
            .current_dir(repo_path)
            .output()?;

        let mut commit_findings = Vec::new();

        for file_path in String::from_utf8_lossy(&files.stdout).lines() {
            if is_example_file(file_path) || should_skip_file(file_path) {
                continue;
            }

            let content = Command::new("git")
                .args(["show", &format!("{}:{}", hash, file_path)])
                .current_dir(repo_path)
                .output();

            let content = match content {
                Ok(c) if c.status.success() => String::from_utf8_lossy(&c.stdout).to_string(),
                _ => continue,
            };

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.starts_with("//") {
                    continue;
                }

                for pattern in PATTERNS {
                    if (pattern.detect)(line) {
                        commit_findings.push(Finding {
                            file: file_path.to_string(),
                            line: line_num + 1,
                            pattern_name: pattern.name.to_string(),
                            preview: mask(line.trim()),
                        });
                        break;
                    }
                }
            }
        }

        if !commit_findings.is_empty() {
            results.push((
                format!("{} — {}", &hash[..8], subject),
                commit_findings,
            ));
        }
    }

    Ok(results)
}
