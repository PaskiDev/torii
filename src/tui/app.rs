use git2::Repository;
use crate::error::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum View {
    Dashboard,
    Diff,
    Log,
    Branch,
    Commit,
    Snapshot,
    Sync,
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

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLineKind {
    Added,
    Removed,
    Context,
    Header,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Panel {
    Staged,
    Unstaged,
    Untracked,
    Log,
}

// ── Dashboard state ──────────────────────────────────────────────────────────

pub struct DashboardState {
    pub selected_panel: Panel,
    pub staged_idx: usize,
    pub unstaged_idx: usize,
    pub untracked_idx: usize,
    pub log_idx: usize,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            selected_panel: Panel::Unstaged,
            staged_idx: 0,
            unstaged_idx: 0,
            untracked_idx: 0,
            log_idx: 0,
        }
    }
}

// ── Diff state ───────────────────────────────────────────────────────────────

pub struct DiffState {
    pub title: String,
    pub lines: Vec<DiffLine>,
    pub scroll: usize,
}

impl Default for DiffState {
    fn default() -> Self {
        Self { title: String::new(), lines: vec![], scroll: 0 }
    }
}

// ── Log state ────────────────────────────────────────────────────────────────

pub struct LogState {
    pub idx: usize,
    pub scroll: usize,
}

impl Default for LogState {
    fn default() -> Self {
        Self { idx: 0, scroll: 0 }
    }
}

// ── Branch state ─────────────────────────────────────────────────────────────

pub struct BranchEntry {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
}

pub struct BranchState {
    pub branches: Vec<BranchEntry>,
    pub idx: usize,
}

impl Default for BranchState {
    fn default() -> Self {
        Self { branches: vec![], idx: 0 }
    }
}

// ── Commit state ─────────────────────────────────────────────────────────────

pub struct CommitState {
    pub message: String,
    pub cursor: usize,
}

impl Default for CommitState {
    fn default() -> Self {
        Self { message: String::new(), cursor: 0 }
    }
}

// ── Snapshot state ───────────────────────────────────────────────────────────

pub struct SnapshotEntry {
    pub id: String,
    pub name: String,
    pub time: String,
}

pub struct SnapshotState {
    pub snapshots: Vec<SnapshotEntry>,
    pub idx: usize,
}

impl Default for SnapshotState {
    fn default() -> Self {
        Self { snapshots: vec![], idx: 0 }
    }
}

// ── Sync state ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum SyncOp {
    PullPush,
    PullOnly,
    PushOnly,
    ForcePush,
    Fetch,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Idle,
    Running,
    Done(String),
    Error(String),
}

pub struct SyncState {
    pub selected_op: SyncOp,
    pub status: SyncStatus,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            selected_op: SyncOp::PullPush,
            status: SyncStatus::Idle,
        }
    }
}

// ── Main App ─────────────────────────────────────────────────────────────────

pub struct App {
    pub should_quit: bool,
    pub view: View,
    pub status_msg: Option<String>,

    // Repo state (shared across views)
    pub repo_path: String,
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,

    // File lists (shared)
    pub staged: Vec<FileEntry>,
    pub unstaged: Vec<FileEntry>,
    pub untracked: Vec<FileEntry>,
    pub commits: Vec<CommitEntry>,

    // Per-view state
    pub dashboard: DashboardState,
    pub diff: DiffState,
    pub log: LogState,
    pub branch_view: BranchState,
    pub commit_view: CommitState,
    pub snapshot_view: SnapshotState,
    pub sync_view: SyncState,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut app = Self {
            should_quit: false,
            view: View::Dashboard,
            status_msg: None,
            repo_path: ".".to_string(),
            branch: String::new(),
            ahead: 0,
            behind: 0,
            staged: vec![],
            unstaged: vec![],
            untracked: vec![],
            commits: vec![],
            dashboard: DashboardState::default(),
            diff: DiffState::default(),
            log: LogState::default(),
            branch_view: BranchState::default(),
            commit_view: CommitState::default(),
            snapshot_view: SnapshotState::default(),
            sync_view: SyncState::default(),
        };
        app.refresh()?;
        Ok(app)
    }

    pub fn go_to(&mut self, view: View) {
        match &view {
            View::Diff => self.load_diff(),
            View::Branch => self.load_branches(),
            View::Snapshot => self.load_snapshots(),
            View::Sync => {
                self.sync_view.status = SyncStatus::Idle;
                self.sync_view.selected_op = SyncOp::PullPush;
            }
            View::Log => {
                self.log.idx = self.dashboard.log_idx;
                self.log.scroll = 0;
            }
            _ => {}
        }
        self.view = view;
        self.status_msg = None;
    }

    pub fn go_back(&mut self) {
        self.view = View::Dashboard;
        self.status_msg = None;
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some(msg.into());
    }

    pub fn refresh(&mut self) -> Result<()> {
        let repo = Repository::discover(&self.repo_path)
            .map_err(crate::error::ToriiError::Git)?;

        self.branch = repo.head().ok()
            .and_then(|h| h.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| "detached".to_string());

        let (ahead, behind) = ahead_behind(&repo, &self.branch).unwrap_or((0, 0));
        self.ahead = ahead;
        self.behind = behind;

        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true);
        let statuses = repo.statuses(Some(&mut opts))
            .map_err(crate::error::ToriiError::Git)?;

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

        self.commits.clear();
        let mut revwalk = repo.revwalk().map_err(crate::error::ToriiError::Git)?;
        let _ = revwalk.push_head();
        for oid in revwalk.take(50) {
            let oid = match oid { Ok(o) => o, Err(_) => continue };
            let commit = match repo.find_commit(oid) { Ok(c) => c, Err(_) => continue };
            let hash = oid.to_string()[..7].to_string();
            let message = commit.summary().unwrap_or("").to_string();
            let author = commit.author().name().unwrap_or("").to_string();
            let time = format_age(commit.time().seconds());
            self.commits.push(CommitEntry { hash, message, author, time });
        }

        Ok(())
    }

    // ── Dashboard helpers ────────────────────────────────────────────────────

    pub fn next_panel(&mut self) {
        self.dashboard.selected_panel = match self.dashboard.selected_panel {
            Panel::Staged    => Panel::Unstaged,
            Panel::Unstaged  => Panel::Untracked,
            Panel::Untracked => Panel::Log,
            Panel::Log       => Panel::Staged,
        };
    }

    pub fn prev_panel(&mut self) {
        self.dashboard.selected_panel = match self.dashboard.selected_panel {
            Panel::Staged    => Panel::Log,
            Panel::Unstaged  => Panel::Staged,
            Panel::Untracked => Panel::Unstaged,
            Panel::Log       => Panel::Untracked,
        };
    }

    pub fn move_up(&mut self) {
        let d = &mut self.dashboard;
        match d.selected_panel {
            Panel::Staged    => { if d.staged_idx > 0    { d.staged_idx -= 1; } }
            Panel::Unstaged  => { if d.unstaged_idx > 0  { d.unstaged_idx -= 1; } }
            Panel::Untracked => { if d.untracked_idx > 0 { d.untracked_idx -= 1; } }
            Panel::Log       => { if d.log_idx > 0       { d.log_idx -= 1; } }
        }
    }

    pub fn move_down(&mut self) {
        let staged_len    = self.staged.len();
        let unstaged_len  = self.unstaged.len();
        let untracked_len = self.untracked.len();
        let commits_len   = self.commits.len();
        let d = &mut self.dashboard;
        match d.selected_panel {
            Panel::Staged    => { if d.staged_idx + 1 < staged_len       { d.staged_idx += 1; } }
            Panel::Unstaged  => { if d.unstaged_idx + 1 < unstaged_len   { d.unstaged_idx += 1; } }
            Panel::Untracked => { if d.untracked_idx + 1 < untracked_len { d.untracked_idx += 1; } }
            Panel::Log       => { if d.log_idx + 1 < commits_len         { d.log_idx += 1; } }
        }
    }

    // ── Diff helpers ─────────────────────────────────────────────────────────

    fn load_diff(&mut self) {
        let panel = &self.dashboard.selected_panel;
        let idx = match panel {
            Panel::Staged    => self.dashboard.staged_idx,
            Panel::Unstaged  => self.dashboard.unstaged_idx,
            Panel::Untracked => self.dashboard.untracked_idx,
            Panel::Log       => { self.load_commit_diff(); return; }
        };

        let files = match panel {
            Panel::Staged    => &self.staged,
            Panel::Unstaged  => &self.unstaged,
            Panel::Untracked => &self.untracked,
            Panel::Log       => unreachable!(),
        };

        if let Some(entry) = files.get(idx) {
            self.diff.title = entry.path.clone();
            self.diff.lines = read_file_diff(&self.repo_path, &entry.path, entry.status == FileStatus::Staged);
            self.diff.scroll = 0;
        }
    }

    fn load_commit_diff(&mut self) {
        let idx = self.dashboard.log_idx;
        if let Some(commit) = self.commits.get(idx) {
            self.diff.title = format!("{} {}", commit.hash, commit.message);
            self.diff.lines = read_commit_diff(&self.repo_path, &commit.hash);
            self.diff.scroll = 0;
        }
    }

    pub fn diff_scroll_up(&mut self) {
        if self.diff.scroll > 0 { self.diff.scroll -= 1; }
    }

    pub fn diff_scroll_down(&mut self) {
        let max = self.diff.lines.len().saturating_sub(1);
        if self.diff.scroll < max { self.diff.scroll += 1; }
    }

    // ── Log helpers ──────────────────────────────────────────────────────────

    pub fn log_move_up(&mut self) {
        if self.log.idx > 0 { self.log.idx -= 1; }
        self.sync_log_scroll();
    }

    pub fn log_move_down(&mut self) {
        if self.log.idx + 1 < self.commits.len() { self.log.idx += 1; }
        self.sync_log_scroll();
    }

    fn sync_log_scroll(&mut self) {
        // Keep selected item visible (page of 20)
        let page = 20usize;
        if self.log.idx < self.log.scroll {
            self.log.scroll = self.log.idx;
        } else if self.log.idx >= self.log.scroll + page {
            self.log.scroll = self.log.idx + 1 - page;
        }
    }

    // ── Branch helpers ───────────────────────────────────────────────────────

    fn load_branches(&mut self) {
        let Ok(repo) = Repository::discover(&self.repo_path) else { return };
        let Ok(branches) = repo.branches(None) else { return };

        self.branch_view.branches.clear();
        for branch in branches.flatten() {
            let (b, btype) = branch;
            let Ok(name) = b.name() else { continue };
            let Some(name) = name else { continue };
            let is_current = b.is_head();
            let is_remote = btype == git2::BranchType::Remote;
            self.branch_view.branches.push(BranchEntry {
                name: name.to_string(),
                is_current,
                is_remote,
            });
        }
        self.branch_view.idx = self.branch_view.branches
            .iter().position(|b| b.is_current).unwrap_or(0);
    }

    pub fn branch_move_up(&mut self) {
        if self.branch_view.idx > 0 { self.branch_view.idx -= 1; }
    }

    pub fn branch_move_down(&mut self) {
        if self.branch_view.idx + 1 < self.branch_view.branches.len() {
            self.branch_view.idx += 1;
        }
    }

    // ── Commit helpers ───────────────────────────────────────────────────────

    pub fn commit_type_char(&mut self, c: char) {
        let cur = self.commit_view.cursor;
        self.commit_view.message.insert(cur, c);
        self.commit_view.cursor += 1;
    }

    pub fn commit_backspace(&mut self) {
        let cur = self.commit_view.cursor;
        if cur > 0 {
            self.commit_view.message.remove(cur - 1);
            self.commit_view.cursor -= 1;
        }
    }

    pub fn commit_cursor_left(&mut self) {
        if self.commit_view.cursor > 0 { self.commit_view.cursor -= 1; }
    }

    pub fn commit_cursor_right(&mut self) {
        let len = self.commit_view.message.len();
        if self.commit_view.cursor < len { self.commit_view.cursor += 1; }
    }

    // ── Snapshot helpers ─────────────────────────────────────────────────────

    fn load_snapshots(&mut self) {
        // Snapshots stored in .git/torii-snapshots/ — read metadata
        self.snapshot_view.snapshots.clear();
        let snap_dir = std::path::Path::new(&self.repo_path)
            .join(".git/torii-snapshots");
        if !snap_dir.exists() { return; }
        if let Ok(entries) = std::fs::read_dir(&snap_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".meta") {
                    let id = name.trim_end_matches(".meta").to_string();
                    let time = entry.metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            let secs = t.duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default().as_secs() as i64;
                            format_age(secs)
                        })
                        .unwrap_or_default();
                    let label = std::fs::read_to_string(entry.path())
                        .unwrap_or_else(|_| id.clone())
                        .trim().to_string();
                    self.snapshot_view.snapshots.push(SnapshotEntry {
                        id: id.clone(),
                        name: label,
                        time,
                    });
                }
            }
        }
        self.snapshot_view.idx = 0;
    }

    pub fn snapshot_move_up(&mut self) {
        if self.snapshot_view.idx > 0 { self.snapshot_view.idx -= 1; }
    }

    pub fn snapshot_move_down(&mut self) {
        if self.snapshot_view.idx + 1 < self.snapshot_view.snapshots.len() {
            self.snapshot_view.idx += 1;
        }
    }

    // ── Sync helpers ─────────────────────────────────────────────────────────

    pub fn sync_op_next(&mut self) {
        self.sync_view.selected_op = match self.sync_view.selected_op {
            SyncOp::PullPush  => SyncOp::PullOnly,
            SyncOp::PullOnly  => SyncOp::PushOnly,
            SyncOp::PushOnly  => SyncOp::ForcePush,
            SyncOp::ForcePush => SyncOp::Fetch,
            SyncOp::Fetch     => SyncOp::PullPush,
        };
    }

    pub fn sync_op_prev(&mut self) {
        self.sync_view.selected_op = match self.sync_view.selected_op {
            SyncOp::PullPush  => SyncOp::Fetch,
            SyncOp::PullOnly  => SyncOp::PullPush,
            SyncOp::PushOnly  => SyncOp::PullOnly,
            SyncOp::ForcePush => SyncOp::PushOnly,
            SyncOp::Fetch     => SyncOp::ForcePush,
        };
    }
}

// ── Git helpers ───────────────────────────────────────────────────────────────

fn ahead_behind(repo: &Repository, branch: &str) -> Option<(usize, usize)> {
    let local  = repo.find_reference(&format!("refs/heads/{}", branch)).ok()?.target()?;
    let remote = repo.find_reference(&format!("refs/remotes/origin/{}", branch)).ok()?.target()?;
    repo.graph_ahead_behind(local, remote).ok()
}

fn read_file_diff(repo_path: &str, file_path: &str, staged: bool) -> Vec<DiffLine> {
    let Ok(repo) = Repository::discover(repo_path) else { return vec![] };
    let mut opts = git2::DiffOptions::new();
    opts.pathspec(file_path);

    let diff = if staged {
        let head = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let tree = head.as_ref().and_then(|c| c.tree().ok());
        let index = repo.index().ok();
        match (tree, index) {
            (Some(t), Some(mut i)) => repo.diff_tree_to_index(Some(&t), Some(&mut i), Some(&mut opts)),
            (None, Some(mut i))    => repo.diff_tree_to_index(None, Some(&mut i), Some(&mut opts)),
            _ => return vec![],
        }
    } else {
        repo.diff_index_to_workdir(None, Some(&mut opts))
    };

    let Ok(diff) = diff else { return vec![] };
    diff_to_lines(&diff)
}

fn read_commit_diff(repo_path: &str, hash: &str) -> Vec<DiffLine> {
    let Ok(repo) = Repository::discover(repo_path) else { return vec![] };
    let Ok(oid) = git2::Oid::from_str(hash) else { return vec![] };
    let Ok(commit) = repo.find_commit(oid) else { return vec![] };
    let Ok(tree) = commit.tree() else { return vec![] };
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());
    let Ok(diff) = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None) else { return vec![] };
    diff_to_lines(&diff)
}

fn diff_to_lines(diff: &git2::Diff) -> Vec<DiffLine> {
    let mut lines = vec![];
    let _ = diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        let content = String::from_utf8_lossy(line.content()).trim_end_matches('\n').to_string();
        let kind = match line.origin() {
            '+' => DiffLineKind::Added,
            '-' => DiffLineKind::Removed,
            'F' | 'H' => DiffLineKind::Header,
            _   => DiffLineKind::Context,
        };
        lines.push(DiffLine { kind, content });
        true
    });
    lines
}

fn format_age(ts: i64) -> String {
    let now = chrono::Utc::now().timestamp();
    let diff = now - ts;
    if diff < 60        { format!("{}s ago", diff) }
    else if diff < 3600 { format!("{}m ago", diff / 60) }
    else if diff < 86400 { format!("{}h ago", diff / 3600) }
    else                { format!("{}d ago", diff / 86400) }
}
