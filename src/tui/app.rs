use git2::Repository;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Dashboard,
    Rebase,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: String,
    pub status: FileStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Staged,
    Unstaged,
    Untracked,
}

#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub time: String,
}

pub struct App {
    pub should_quit: bool,
    pub view: View,

    // Repo state
    pub repo_path: String,
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,

    // File lists
    pub staged: Vec<FileEntry>,
    pub unstaged: Vec<FileEntry>,
    pub untracked: Vec<FileEntry>,

    // Log
    pub commits: Vec<CommitEntry>,

    // Selection state
    pub selected_panel: Panel,
    pub staged_idx: usize,
    pub unstaged_idx: usize,
    pub untracked_idx: usize,
    pub log_idx: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Staged,
    Unstaged,
    Untracked,
    Log,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut app = Self {
            should_quit: false,
            view: View::Dashboard,
            repo_path: ".".to_string(),
            branch: String::new(),
            ahead: 0,
            behind: 0,
            staged: vec![],
            unstaged: vec![],
            untracked: vec![],
            commits: vec![],
            selected_panel: Panel::Unstaged,
            staged_idx: 0,
            unstaged_idx: 0,
            untracked_idx: 0,
            log_idx: 0,
        };
        app.refresh()?;
        Ok(app)
    }

    pub fn refresh(&mut self) -> Result<()> {
        let repo = Repository::discover(&self.repo_path)
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        // Branch
        self.branch = repo.head().ok()
            .and_then(|h| h.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| "detached".to_string());

        // Ahead/behind
        let (ahead, behind) = ahead_behind(&repo, &self.branch).unwrap_or((0, 0));
        self.ahead = ahead;
        self.behind = behind;

        // Status
        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts))
            .map_err(|e| crate::error::ToriiError::Git(e))?;

        self.staged.clear();
        self.unstaged.clear();
        self.untracked.clear();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let s = entry.status();

            if s.intersects(
                git2::Status::INDEX_NEW | git2::Status::INDEX_MODIFIED |
                git2::Status::INDEX_DELETED | git2::Status::INDEX_RENAMED
            ) {
                self.staged.push(FileEntry { path: path.clone(), status: FileStatus::Staged });
            }
            if s.intersects(
                git2::Status::WT_MODIFIED | git2::Status::WT_DELETED | git2::Status::WT_RENAMED
            ) {
                self.unstaged.push(FileEntry { path: path.clone(), status: FileStatus::Unstaged });
            }
            if s.contains(git2::Status::WT_NEW) {
                self.untracked.push(FileEntry { path, status: FileStatus::Untracked });
            }
        }

        // Log — last 20 commits
        self.commits.clear();
        let mut revwalk = repo.revwalk()
            .map_err(|e| crate::error::ToriiError::Git(e))?;
        let _ = revwalk.push_head();
        for oid in revwalk.take(20) {
            let oid = match oid { Ok(o) => o, Err(_) => continue };
            let commit = match repo.find_commit(oid) { Ok(c) => c, Err(_) => continue };
            let hash = oid.to_string()[..7].to_string();
            let message = commit.summary().unwrap_or("").to_string();
            let author = commit.author().name().unwrap_or("").to_string();
            let ts = commit.time().seconds();
            let time = format_age(ts);
            self.commits.push(CommitEntry { hash, message, author, time });
        }

        Ok(())
    }

    pub fn next_panel(&mut self) {
        self.selected_panel = match self.selected_panel {
            Panel::Staged => Panel::Unstaged,
            Panel::Unstaged => Panel::Untracked,
            Panel::Untracked => Panel::Log,
            Panel::Log => Panel::Staged,
        };
    }

    pub fn prev_panel(&mut self) {
        self.selected_panel = match self.selected_panel {
            Panel::Staged => Panel::Log,
            Panel::Unstaged => Panel::Staged,
            Panel::Untracked => Panel::Unstaged,
            Panel::Log => Panel::Untracked,
        };
    }

    pub fn move_up(&mut self) {
        match self.selected_panel {
            Panel::Staged    => { if self.staged_idx > 0 { self.staged_idx -= 1; } }
            Panel::Unstaged  => { if self.unstaged_idx > 0 { self.unstaged_idx -= 1; } }
            Panel::Untracked => { if self.untracked_idx > 0 { self.untracked_idx -= 1; } }
            Panel::Log       => { if self.log_idx > 0 { self.log_idx -= 1; } }
        }
    }

    pub fn move_down(&mut self) {
        match self.selected_panel {
            Panel::Staged    => { if self.staged_idx + 1 < self.staged.len() { self.staged_idx += 1; } }
            Panel::Unstaged  => { if self.unstaged_idx + 1 < self.unstaged.len() { self.unstaged_idx += 1; } }
            Panel::Untracked => { if self.untracked_idx + 1 < self.untracked.len() { self.untracked_idx += 1; } }
            Panel::Log       => { if self.log_idx + 1 < self.commits.len() { self.log_idx += 1; } }
        }
    }
}

fn ahead_behind(repo: &Repository, branch: &str) -> Option<(usize, usize)> {
    let local = repo.find_reference(&format!("refs/heads/{}", branch)).ok()?.target()?;
    let remote = repo.find_reference(&format!("refs/remotes/origin/{}", branch)).ok()?.target()?;
    repo.graph_ahead_behind(local, remote).ok()
}

fn format_age(ts: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - ts;
    if diff < 60 { format!("{}s ago", diff) }
    else if diff < 3600 { format!("{}m ago", diff / 60) }
    else if diff < 86400 { format!("{}h ago", diff / 3600) }
    else { format!("{}d ago", diff / 86400) }
}
