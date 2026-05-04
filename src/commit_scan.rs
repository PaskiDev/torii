//! Commit policy enforcement.
//!
//! Loads a `policies/commits.gate` file, parses it via the `gate` crate, and
//! evaluates each commit in a range against the rules. Used by:
//!
//!   - `torii scan commits [--all] [--since N]` (CLI)
//!   - pre-save hook (planned)
//!   - CI gates (future, server-side)
//!
//! The Gate DSL is parsed once into `PolicyDecl` rules; we then compile the
//! regex patterns and run them per commit. Rule failures are returned as
//! `Violation`s so callers can format / surface them however they want.

use std::path::{Path, PathBuf};

use gate::gate::ast::{Item, PolicyDecl, PolicyKind, PolicyRule};
use gate::gate::lexer::Lexer;
use gate::gate::parser::Parser;
use regex::Regex;

use crate::error::{Result, ToriiError};

/// Compiled, ready-to-run version of a `policy commits {}` block.
pub struct CompiledCommitPolicy {
    forbid_trailers: Vec<Regex>,
    require_trailers: Vec<Regex>,
    forbid_subjects: Vec<Regex>,
    author_email_matches: Option<Regex>,
    subject_max_length: Option<usize>,
    subject_min_length: Option<usize>,
    require_conventional: bool,
}

/// One rule failure for one commit.
#[derive(Debug, Clone)]
pub struct Violation {
    /// Full commit OID — kept for future `--fix` mode that needs to
    /// rewrite the exact commit.
    #[allow(dead_code)]
    pub commit_id: String,
    pub commit_short: String,
    pub subject: String,
    pub rule: String,
    pub detail: String,
}

impl CompiledCommitPolicy {
    pub fn from_decl(decl: &PolicyDecl) -> Result<Self> {
        if decl.kind != PolicyKind::Commits {
            return Err(ToriiError::InvalidConfig(
                "expected `policy commits` block".into(),
            ));
        }
        let mut p = CompiledCommitPolicy {
            forbid_trailers: Vec::new(),
            require_trailers: Vec::new(),
            forbid_subjects: Vec::new(),
            author_email_matches: None,
            subject_max_length: None,
            subject_min_length: None,
            require_conventional: false,
        };
        for rule in &decl.rules {
            match rule {
                PolicyRule::ForbidTrailer(pat) => {
                    p.forbid_trailers.push(compile(pat)?);
                }
                PolicyRule::RequireTrailer(pat) => {
                    p.require_trailers.push(compile(pat)?);
                }
                PolicyRule::ForbidSubject(pat) => {
                    p.forbid_subjects.push(compile(pat)?);
                }
                PolicyRule::AuthorEmailMatches(pat) => {
                    p.author_email_matches = Some(compile(pat)?);
                }
                PolicyRule::SubjectMaxLength(n) => {
                    p.subject_max_length = Some(*n);
                }
                PolicyRule::SubjectMinLength(n) => {
                    p.subject_min_length = Some(*n);
                }
                PolicyRule::ConventionalCommitsRequired => {
                    p.require_conventional = true;
                }
            }
        }
        Ok(p)
    }

    /// Load + compile a policy file. Returns `Ok(None)` if file does not
    /// exist (callers can decide whether absent policy is silent or an error).
    pub fn load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }
        let src = std::fs::read_to_string(path)
            .map_err(|e| ToriiError::InvalidConfig(format!("read {}: {}", path.display(), e)))?;
        let tokens = Lexer::new(&src)
            .tokenize()
            .map_err(|e| ToriiError::InvalidConfig(format!("lex {}: {:?}", path.display(), e)))?;
        let prog = Parser::new(tokens)
            .parse()
            .map_err(|e| ToriiError::InvalidConfig(format!("parse {}: {:?}", path.display(), e)))?;
        let decl = prog.items.iter().find_map(|i| match i {
            Item::Policy(d) if d.kind == PolicyKind::Commits => Some(d.clone()),
            _ => None,
        });
        match decl {
            Some(d) => Ok(Some(Self::from_decl(&d)?)),
            None => Ok(None),
        }
    }

    /// Evaluate a commit. Returns 0+ violations.
    pub fn check(
        &self,
        commit_id: &str,
        author_email: &str,
        message: &str,
    ) -> Vec<Violation> {
        let short: String = commit_id.chars().take(7).collect();
        let subject = message.lines().next().unwrap_or("").trim().to_string();
        let mut out = Vec::new();

        let push = |out: &mut Vec<Violation>, rule: &str, detail: String| {
            out.push(Violation {
                commit_id: commit_id.to_string(),
                commit_short: short.clone(),
                subject: subject.clone(),
                rule: rule.to_string(),
                detail,
            });
        };

        for re in &self.forbid_trailers {
            for line in message.lines() {
                if re.is_match(line) {
                    push(
                        &mut out,
                        "forbid trailer",
                        format!("matches /{}/: `{}`", re.as_str(), line.trim()),
                    );
                    break; // one match per rule per commit is enough
                }
            }
        }

        for re in &self.require_trailers {
            let found = message.lines().any(|l| re.is_match(l));
            if !found {
                push(
                    &mut out,
                    "require trailer",
                    format!("no line matches /{}/", re.as_str()),
                );
            }
        }

        for re in &self.forbid_subjects {
            if re.is_match(&subject) {
                push(
                    &mut out,
                    "forbid subject",
                    format!("subject matches /{}/", re.as_str()),
                );
            }
        }

        if let Some(re) = &self.author_email_matches {
            if !re.is_match(author_email) {
                push(
                    &mut out,
                    "author email",
                    format!("`{}` doesn't match /{}/", author_email, re.as_str()),
                );
            }
        }

        if let Some(max) = self.subject_max_length {
            let len = subject.chars().count();
            if len > max {
                push(
                    &mut out,
                    "subject max_length",
                    format!("subject is {} chars (max {})", len, max),
                );
            }
        }
        if let Some(min) = self.subject_min_length {
            let len = subject.chars().count();
            if len < min {
                push(
                    &mut out,
                    "subject min_length",
                    format!("subject is {} chars (min {})", len, min),
                );
            }
        }

        if self.require_conventional && !is_conventional(&subject) {
            push(
                &mut out,
                "conventional_commits",
                format!("subject doesn't match `<type>(scope?): description`"),
            );
        }

        out
    }
}

fn compile(pat: &str) -> Result<Regex> {
    // Default to case-insensitive — trailers / authors are usually compared
    // without caring about case.
    let with_flag = format!("(?i){}", pat);
    Regex::new(&with_flag)
        .map_err(|e| ToriiError::InvalidConfig(format!("bad regex /{}/: {}", pat, e)))
}

/// Conventional Commits subject:
///   feat: ...
///   feat(scope): ...
///   fix!: ...   (breaking)
///   chore(release)!: ...
fn is_conventional(subject: &str) -> bool {
    static TYPES: &[&str] = &[
        "feat", "fix", "docs", "style", "refactor", "perf", "test",
        "build", "ci", "chore", "revert",
    ];
    let Some(colon) = subject.find(':') else { return false };
    let head = &subject[..colon];
    let head = head.strip_suffix('!').unwrap_or(head);
    let (ty, _scope) = match head.find('(') {
        Some(open) => {
            let close = head.rfind(')').unwrap_or(open);
            (&head[..open], Some(&head[open + 1..close]))
        }
        None => (head, None),
    };
    TYPES.contains(&ty)
}

/// Default location of the commit policy file inside a repo.
pub fn default_policy_path(repo_root: &Path) -> PathBuf {
    repo_root.join("policies").join("commits.gate")
}

/// Convenience: scan a range of commits with a loaded policy.
/// `since_oid` = inclusive end of range (older). If None, walks all of HEAD.
pub fn scan_repo(
    repo: &git2::Repository,
    policy: &CompiledCommitPolicy,
    limit: usize,
) -> Result<Vec<Violation>> {
    let mut walk = repo.revwalk().map_err(ToriiError::Git)?;
    walk.push_head().map_err(ToriiError::Git)?;
    let mut all = Vec::new();
    for oid in walk.take(limit) {
        let oid = oid.map_err(ToriiError::Git)?;
        let commit = repo.find_commit(oid).map_err(ToriiError::Git)?;
        let id = oid.to_string();
        let email = commit.author().email().unwrap_or("").to_string();
        let msg = commit.message().unwrap_or("").to_string();
        all.extend(policy.check(&id, &email, &msg));
    }
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pol(src: &str) -> CompiledCommitPolicy {
        let tokens = Lexer::new(src).tokenize().unwrap();
        let prog = Parser::new(tokens).parse().unwrap();
        let Item::Policy(d) = &prog.items[0] else { panic!() };
        CompiledCommitPolicy::from_decl(d).unwrap()
    }

    #[test]
    fn forbid_trailer_catches_claude() {
        let p = pol("policy commits { forbid trailer /Co-Authored-By:.*Claude/ }");
        let v = p.check(
            "abc123",
            "x@y",
            "feat: stuff\n\nCo-Authored-By: Claude Sonnet <noreply@anthropic.com>",
        );
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, "forbid trailer");
    }

    #[test]
    fn require_trailer_missing() {
        let p = pol("policy commits { require trailer /Signed-off-by:/ }");
        let v = p.check("abc", "x@y", "feat: stuff");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].rule, "require trailer");
    }

    #[test]
    fn require_trailer_present_no_violation() {
        let p = pol("policy commits { require trailer /Signed-off-by:/ }");
        let v = p.check("abc", "x@y", "feat: stuff\n\nSigned-off-by: A B <a@b>");
        assert!(v.is_empty());
    }

    #[test]
    fn subject_length_limits() {
        let p = pol("policy commits { subject max_length 10  subject min_length 5 }");
        assert_eq!(p.check("a", "x@y", "ok done").len(), 0);
        assert_eq!(p.check("a", "x@y", "x").len(), 1); // too short
        assert_eq!(p.check("a", "x@y", "way too long subject here").len(), 1); // too long
    }

    #[test]
    fn forbid_subject() {
        let p = pol("policy commits { forbid subject /^(wip|tmp)$/ }");
        assert_eq!(p.check("a", "x@y", "wip").len(), 1);
        assert_eq!(p.check("a", "x@y", "feat: real").len(), 0);
    }

    #[test]
    fn author_email_mismatch() {
        let p = pol(r#"policy commits { author email matches /.*@paski\.dev$/ }"#);
        assert_eq!(p.check("a", "x@y.com", "feat: x").len(), 1);
        assert_eq!(p.check("a", "me@paski.dev", "feat: x").len(), 0);
    }

    #[test]
    fn conventional_commits() {
        let p = pol("policy commits { conventional_commits required }");
        assert_eq!(p.check("a", "x@y", "feat: ok").len(), 0);
        assert_eq!(p.check("a", "x@y", "feat(scope): ok").len(), 0);
        assert_eq!(p.check("a", "x@y", "fix!: breaking").len(), 0);
        assert_eq!(p.check("a", "x@y", "random message").len(), 1);
        assert_eq!(p.check("a", "x@y", "wibble: unknown type").len(), 1);
    }

    #[test]
    fn is_conventional_helper() {
        assert!(is_conventional("feat: x"));
        assert!(is_conventional("feat(scope): x"));
        assert!(is_conventional("fix!: x"));
        assert!(is_conventional("chore(release)!: x"));
        assert!(!is_conventional("random"));
        assert!(!is_conventional("frob: x"));
    }
}
