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
    Tag,
    History,
    Remote,
    Mirror,
    Workspace,
    Config,
    Settings,
    Help,
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

#[derive(Debug, Clone, PartialEq)]
pub enum CommitFocus {
    List,
    Input,
}

pub struct CommitState {
    pub message: String,
    pub cursor: usize,
    pub focus: CommitFocus,
}

impl Default for CommitState {
    fn default() -> Self {
        Self { message: String::new(), cursor: 0, focus: CommitFocus::List }
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

// ── Tag state ────────────────────────────────────────────────────────────────

pub struct TagEntry {
    pub name: String,
    pub message: String,
    pub time: String,
}

pub struct TagState {
    pub tags: Vec<TagEntry>,
    pub idx: usize,
    pub status: Option<String>,
}

impl Default for TagState {
    fn default() -> Self { Self { tags: vec![], idx: 0, status: None } }
}

// ── History state ─────────────────────────────────────────────────────────────

pub struct ReflogEntry {
    pub id: String,
    pub message: String,
    pub time: String,
}

pub struct HistoryState {
    pub reflog: Vec<ReflogEntry>,
    pub idx: usize,
    pub status: Option<String>,
}

impl Default for HistoryState {
    fn default() -> Self { Self { reflog: vec![], idx: 0, status: None } }
}

// ── Remote state ──────────────────────────────────────────────────────────────

pub struct RemoteEntry {
    pub name: String,
    pub url: String,
}

pub struct RemoteState {
    pub remotes: Vec<RemoteEntry>,
    pub idx: usize,
    pub status: Option<String>,
}

impl Default for RemoteState {
    fn default() -> Self { Self { remotes: vec![], idx: 0, status: None } }
}

// ── Mirror state ──────────────────────────────────────────────────────────────

pub struct MirrorEntry {
    pub name: String,
    pub url: String,
    pub kind: String,
}

pub struct MirrorState {
    pub mirrors: Vec<MirrorEntry>,
    pub idx: usize,
    pub status: Option<String>,
}

impl Default for MirrorState {
    fn default() -> Self { Self { mirrors: vec![], idx: 0, status: None } }
}

// ── Workspace state ───────────────────────────────────────────────────────────

pub struct WorkspaceRepo {
    pub path: String,
    pub branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub dirty: bool,
}

pub struct WorkspaceEntry {
    pub name: String,
    pub repos: Vec<WorkspaceRepo>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceFocus { Workspaces, Repos }

pub struct WorkspaceState {
    pub workspaces: Vec<WorkspaceEntry>,
    pub ws_idx: usize,
    pub repo_idx: usize,
    pub focus: WorkspaceFocus,
    pub status: Option<String>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self { workspaces: vec![], ws_idx: 0, repo_idx: 0, focus: WorkspaceFocus::Workspaces, status: None }
    }
}

// ── Config state ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigScope { Global, Local }

pub struct ConfigEntry {
    pub key: String,
    pub value: String,
    pub scope: ConfigScope,
    pub section: String,
}

pub struct ConfigState {
    pub entries: Vec<ConfigEntry>,
    pub idx: usize,
    pub editing: bool,
    pub edit_buf: String,
    pub edit_cursor: usize,
    pub scope: ConfigScope,
    pub status: Option<String>,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            entries: vec![],
            idx: 0,
            editing: false,
            edit_buf: String::new(),
            edit_cursor: 0,
            scope: ConfigScope::Global,
            status: None,
        }
    }
}

// ── Settings state ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum BorderStyle { Rounded, Sharp }

#[derive(Debug, Clone)]
pub struct TuiSettings {
    pub border_style: BorderStyle,
    pub show_help_view: bool,
    pub show_history_view: bool,
    pub show_mirror_view: bool,
    pub show_workspace_view: bool,
    pub show_remote_view: bool,
    pub keybind_files: char,
    pub keybind_save: char,
    pub keybind_sync: char,
    pub keybind_snapshot: char,
    pub keybind_log: char,
    pub keybind_branch: char,
    pub keybind_tag: char,
    pub keybind_history: char,
    pub keybind_remote: char,
    pub keybind_mirror: char,
    pub keybind_workspace: char,
    pub keybind_config: char,
    pub brand_color: (u8, u8, u8),
    pub selected_bg: (u8, u8, u8),
    pub event_log_max: usize,
}

impl Default for TuiSettings {
    fn default() -> Self {
        Self {
            border_style: BorderStyle::Rounded,
            show_help_view: true,
            show_history_view: true,
            show_mirror_view: true,
            show_workspace_view: true,
            show_remote_view: true,
            keybind_files: 'f',
            keybind_save: 'c',
            keybind_sync: 's',
            keybind_snapshot: 'p',
            keybind_log: 'l',
            keybind_branch: 'b',
            keybind_tag: 't',
            keybind_history: 'h',
            keybind_remote: 'r',
            keybind_mirror: 'm',
            keybind_workspace: 'w',
            keybind_config: 'g',
            brand_color: (255, 76, 76),
            selected_bg: (40, 40, 60),
            event_log_max: 50,
        }
    }
}

impl TuiSettings {
    pub fn load() -> Self {
        let path = dirs::home_dir()
            .map(|h| h.join(".torii/tui-settings.toml"))
            .unwrap_or_default();
        if !path.exists() { return Self::default(); }
        let Ok(content) = std::fs::read_to_string(&path) else { return Self::default(); };
        let mut s = Self::default();
        for line in content.lines() {
            let line = line.trim();
            let mut parts = line.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let val = parts.next().unwrap_or("").trim().trim_matches('"');
            match key {
                "border_style"       => s.border_style = if val == "sharp" { BorderStyle::Sharp } else { BorderStyle::Rounded },
                "show_help_view"     => s.show_help_view = val != "false",
                "show_history_view"  => s.show_history_view = val != "false",
                "show_mirror_view"   => s.show_mirror_view = val != "false",
                "show_workspace_view"=> s.show_workspace_view = val != "false",
                "show_remote_view"   => s.show_remote_view = val != "false",
                "keybind_files"      => if let Some(c) = val.chars().next() { s.keybind_files = c; }
                "keybind_save"       => if let Some(c) = val.chars().next() { s.keybind_save = c; }
                "keybind_sync"       => if let Some(c) = val.chars().next() { s.keybind_sync = c; }
                "keybind_snapshot"   => if let Some(c) = val.chars().next() { s.keybind_snapshot = c; }
                "keybind_log"        => if let Some(c) = val.chars().next() { s.keybind_log = c; }
                "keybind_branch"     => if let Some(c) = val.chars().next() { s.keybind_branch = c; }
                "keybind_tag"        => if let Some(c) = val.chars().next() { s.keybind_tag = c; }
                "keybind_history"    => if let Some(c) = val.chars().next() { s.keybind_history = c; }
                "keybind_remote"     => if let Some(c) = val.chars().next() { s.keybind_remote = c; }
                "keybind_mirror"     => if let Some(c) = val.chars().next() { s.keybind_mirror = c; }
                "keybind_workspace"  => if let Some(c) = val.chars().next() { s.keybind_workspace = c; }
                "keybind_config"     => if let Some(c) = val.chars().next() { s.keybind_config = c; }
                "brand_color"        => { if let Some(rgb) = parse_rgb(val) { s.brand_color = rgb; } }
                "selected_bg"        => { if let Some(rgb) = parse_rgb(val) { s.selected_bg = rgb; } }
                "event_log_max"      => { if let Ok(n) = val.parse::<usize>() { s.event_log_max = n; } }
                _ => {}
            }
        }
        s
    }

    pub fn save(&self) {
        let path = dirs::home_dir()
            .map(|h| h.join(".torii/tui-settings.toml"))
            .unwrap_or_default();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = format!(
            "border_style = \"{}\"\nshow_help_view = {}\nshow_history_view = {}\nshow_mirror_view = {}\nshow_workspace_view = {}\nshow_remote_view = {}\nkeybind_files = \"{}\"\nkeybind_save = \"{}\"\nkeybind_sync = \"{}\"\nkeybind_snapshot = \"{}\"\nkeybind_log = \"{}\"\nkeybind_branch = \"{}\"\nkeybind_tag = \"{}\"\nkeybind_history = \"{}\"\nkeybind_remote = \"{}\"\nkeybind_mirror = \"{}\"\nkeybind_workspace = \"{}\"\nkeybind_config = \"{}\"\nbrand_color = \"{},{},{}\"\nselected_bg = \"{},{},{}\"\nevent_log_max = {}\n",
            if self.border_style == BorderStyle::Rounded { "rounded" } else { "sharp" },
            self.show_help_view, self.show_history_view, self.show_mirror_view,
            self.show_workspace_view, self.show_remote_view,
            self.keybind_files, self.keybind_save, self.keybind_sync,
            self.keybind_snapshot, self.keybind_log, self.keybind_branch,
            self.keybind_tag, self.keybind_history, self.keybind_remote,
            self.keybind_mirror, self.keybind_workspace, self.keybind_config,
            self.brand_color.0, self.brand_color.1, self.brand_color.2,
            self.selected_bg.0, self.selected_bg.1, self.selected_bg.2,
            self.event_log_max,
        );
        let _ = std::fs::write(path, content);
    }
}

fn parse_rgb(s: &str) -> Option<(u8, u8, u8)> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 { return None; }
    Some((
        parts[0].trim().parse().ok()?,
        parts[1].trim().parse().ok()?,
        parts[2].trim().parse().ok()?,
    ))
}

pub struct SettingsState {
    pub idx: usize,
    pub editing_keybind: Option<usize>,
    pub status: Option<String>,
}

impl Default for SettingsState {
    fn default() -> Self { Self { idx: 0, editing_keybind: None, status: None } }
}

#[derive(Clone, PartialEq)]
pub enum EventKind { Error, Success, Info }

#[derive(Clone)]
pub struct EventEntry {
    pub timestamp: String,
    pub message: String,
    pub kind: EventKind,
}

// ── Main App ─────────────────────────────────────────────────────────────────

pub struct App {
    pub should_quit: bool,
    pub view: View,
    pub sidebar_idx: usize,
    pub sidebar_focused: bool,
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
    pub tag_view: TagState,
    pub history_view: HistoryState,
    pub remote_view: RemoteState,
    pub mirror_view: MirrorState,
    pub workspace_view: WorkspaceState,
    pub config_view: ConfigState,
    pub settings_view: SettingsState,
    pub settings: TuiSettings,

    pub event_log: Vec<EventEntry>,
    pub show_event_log: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let mut app = Self {
            should_quit: false,
            view: View::Dashboard,
            sidebar_idx: 0,
            sidebar_focused: true,
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
            tag_view: TagState::default(),
            history_view: HistoryState::default(),
            remote_view: RemoteState::default(),
            mirror_view: MirrorState::default(),
            workspace_view: WorkspaceState::default(),
            config_view: ConfigState::default(),
            settings_view: SettingsState::default(),
            settings: TuiSettings::load(),
            event_log: vec![],
            show_event_log: false,
        };
        app.refresh()?;
        Ok(app)
    }

    pub fn sidebar_up(&mut self) {
        if self.sidebar_idx > 0 { self.sidebar_idx -= 1; }
    }

    pub fn sidebar_down(&mut self) {
        if self.sidebar_idx < 12 { self.sidebar_idx += 1; }
    }

    pub fn sidebar_enter(&mut self) {
        let view = match self.sidebar_idx {
            0  => View::Dashboard,
            1  => View::Commit,
            2  => View::Sync,
            3  => View::Snapshot,
            4  => View::Log,
            5  => View::Branch,
            6  => View::Tag,
            7  => View::History,
            8  => View::Remote,
            9  => View::Mirror,
            10 => View::Workspace,
            11 => View::Config,
            12 => View::Settings,
            _  => View::Dashboard,
        };
        self.go_to(view);
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
            View::Tag       => self.load_tags(),
            View::History   => self.load_reflog(),
            View::Remote    => self.load_remotes(),
            View::Mirror    => self.load_mirrors(),
            View::Workspace => self.load_workspaces(),
            View::Config    => self.load_config(),
            _ => {}
        }
        self.sidebar_idx = match &view {
            View::Dashboard => 0,
            View::Commit    => 1,
            View::Sync      => 2,
            View::Snapshot  => 3,
            View::Log       => 4,
            View::Branch    => 5,
            View::Tag       => 6,
            View::History   => 7,
            View::Remote    => 8,
            View::Mirror    => 9,
            View::Workspace => 10,
            View::Config    => 11,
            View::Settings  => 12,
            _               => self.sidebar_idx,
        };
        self.view = view;
        self.status_msg = None;
    }

    pub fn go_back(&mut self) {
        self.view = View::Dashboard;
        self.sidebar_idx = 0;
        self.sidebar_focused = true;
        self.status_msg = None;
    }

    pub fn border_type(&self) -> ratatui::widgets::BorderType {
        if self.settings.border_style == BorderStyle::Rounded {
            ratatui::widgets::BorderType::Rounded
        } else {
            ratatui::widgets::BorderType::Plain
        }
    }

    pub fn brand_color(&self) -> ratatui::style::Color {
        let (r, g, b) = self.settings.brand_color;
        ratatui::style::Color::Rgb(r, g, b)
    }

    pub fn selected_bg(&self) -> ratatui::style::Color {
        let (r, g, b) = self.settings.selected_bg;
        ratatui::style::Color::Rgb(r, g, b)
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some(msg.into());
    }

    pub fn log_event(&mut self, msg: impl Into<String>, kind: EventKind) {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let hh = (secs % 86400) / 3600;
        let mm = (secs % 3600) / 60;
        let ss = secs % 60;
        self.event_log.insert(0, EventEntry {
            timestamp: format!("{:02}:{:02}:{:02}", hh, mm, ss),
            message: msg.into(),
            kind,
        });
        let max = self.settings.event_log_max;
        if self.event_log.len() > max {
            self.event_log.truncate(max);
        }
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

    // Tab cycle: sidebar → view panels → sidebar
    // Returns true if we wrapped back to sidebar
    pub fn tab_cycle(&mut self) -> bool {
        if self.sidebar_focused {
            self.sidebar_focused = false;
            // Enter first panel of current view
            match self.view {
                View::Dashboard => self.dashboard.selected_panel = Panel::Unstaged,
                View::Workspace => self.workspace_view.focus = WorkspaceFocus::Workspaces,
                View::Commit    => self.commit_view.focus = CommitFocus::List,
                _ => {}
            }
            return false;
        }
        // Cycle within view, wrap to sidebar when exhausted
        match self.view {
            View::Dashboard => {
                self.dashboard.selected_panel = match self.dashboard.selected_panel {
                    Panel::Unstaged  => Panel::Untracked,
                    Panel::Untracked => Panel::Staged,
                    Panel::Staged    => Panel::Log,
                    Panel::Log       => { self.sidebar_focused = true; return true; }
                };
            }
            View::Workspace => {
                match self.workspace_view.focus {
                    WorkspaceFocus::Workspaces => self.workspace_view.focus = WorkspaceFocus::Repos,
                    WorkspaceFocus::Repos      => { self.sidebar_focused = true; return true; }
                }
            }
            View::Commit => {
                match self.commit_view.focus {
                    CommitFocus::List  => self.commit_view.focus = CommitFocus::Input,
                    CommitFocus::Input => { self.sidebar_focused = true; return true; }
                }
            }
            _ => { self.sidebar_focused = true; return true; }
        }
        false
    }

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

    // ── Tag helpers ──────────────────────────────────────────────────────────

    fn load_tags(&mut self) {
        self.tag_view.tags.clear();
        let Ok(repo) = Repository::discover(&self.repo_path) else { return };
        let _ = repo.tag_foreach(|oid, name| {
            let name = String::from_utf8_lossy(name).to_string();
            let name = name.trim_start_matches("refs/tags/").to_string();
            let (message, time) = repo.find_object(oid, None).ok()
                .and_then(|obj| obj.peel_to_commit().ok())
                .map(|c| (
                    c.summary().unwrap_or("").to_string(),
                    format_age(c.time().seconds()),
                ))
                .unwrap_or_default();
            self.tag_view.tags.push(TagEntry { name, message, time });
            true
        });
        self.tag_view.idx = 0;
    }

    pub fn tag_move_up(&mut self) {
        if self.tag_view.idx > 0 { self.tag_view.idx -= 1; }
    }

    pub fn tag_move_down(&mut self) {
        if self.tag_view.idx + 1 < self.tag_view.tags.len() { self.tag_view.idx += 1; }
    }

    // ── History helpers ──────────────────────────────────────────────────────

    fn load_reflog(&mut self) {
        self.history_view.reflog.clear();
        let Ok(repo) = Repository::discover(&self.repo_path) else { return };
        let Ok(reflog) = repo.reflog("HEAD") else { return };
        for entry in reflog.iter() {
            let id = entry.id_new().to_string()[..7].to_string();
            let message = entry.message().unwrap_or("").to_string();
            let time = format_age(entry.committer().when().seconds());
            self.history_view.reflog.push(ReflogEntry { id, message, time });
        }
        self.history_view.idx = 0;
    }

    pub fn history_move_up(&mut self) {
        if self.history_view.idx > 0 { self.history_view.idx -= 1; }
    }

    pub fn history_move_down(&mut self) {
        if self.history_view.idx + 1 < self.history_view.reflog.len() {
            self.history_view.idx += 1;
        }
    }

    // ── Remote helpers ───────────────────────────────────────────────────────

    fn load_remotes(&mut self) {
        self.remote_view.remotes.clear();
        let Ok(repo) = Repository::discover(&self.repo_path) else { return };
        let Ok(remotes) = repo.remotes() else { return };
        for name in remotes.iter().flatten() {
            let url = repo.find_remote(name)
                .ok()
                .and_then(|r| r.url().map(|u| u.to_string()))
                .unwrap_or_default();
            self.remote_view.remotes.push(RemoteEntry { name: name.to_string(), url });
        }
        self.remote_view.idx = 0;
    }

    pub fn remote_move_up(&mut self) {
        if self.remote_view.idx > 0 { self.remote_view.idx -= 1; }
    }

    pub fn remote_move_down(&mut self) {
        if self.remote_view.idx + 1 < self.remote_view.remotes.len() {
            self.remote_view.idx += 1;
        }
    }

    // ── Mirror helpers ───────────────────────────────────────────────────────

    fn load_mirrors(&mut self) {
        self.mirror_view.mirrors.clear();
        // Mirrors stored in .torii/mirrors.toml
        let mirrors_path = std::path::Path::new(&self.repo_path).join(".torii/mirrors.toml");
        if !mirrors_path.exists() { return; }
        let Ok(content) = std::fs::read_to_string(&mirrors_path) else { return };
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("url") {
                let url = line.split('=').nth(1).unwrap_or("").trim().trim_matches('"').to_string();
                self.mirror_view.mirrors.push(MirrorEntry {
                    name: format!("mirror-{}", self.mirror_view.mirrors.len() + 1),
                    url,
                    kind: "replica".to_string(),
                });
            }
        }
        self.mirror_view.idx = 0;
    }

    pub fn mirror_move_up(&mut self) {
        if self.mirror_view.idx > 0 { self.mirror_view.idx -= 1; }
    }

    pub fn mirror_move_down(&mut self) {
        if self.mirror_view.idx + 1 < self.mirror_view.mirrors.len() {
            self.mirror_view.idx += 1;
        }
    }

    // ── Workspace helpers ────────────────────────────────────────────────────

    fn load_workspaces(&mut self) {
        self.workspace_view.workspaces.clear();
        let ws_path = dirs::home_dir()
            .map(|h| h.join(".torii/workspaces.toml"))
            .unwrap_or_default();
        if !ws_path.exists() { return; }
        let Ok(content) = std::fs::read_to_string(&ws_path) else { return };
        let mut current_ws: Option<WorkspaceEntry> = None;
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') && line.ends_with(']') {
                if let Some(ws) = current_ws.take() {
                    self.workspace_view.workspaces.push(ws);
                }
                let name = line.trim_matches(|c| c == '[' || c == ']').to_string();
                current_ws = Some(WorkspaceEntry { name, repos: vec![] });
            } else if line.starts_with("path") {
                if let Some(ws) = current_ws.as_mut() {
                    let path = line.split('=').nth(1).unwrap_or("").trim().trim_matches('"').to_string();
                    let (branch, ahead, behind, dirty) = repo_quick_status(&path);
                    ws.repos.push(WorkspaceRepo { path, branch, ahead, behind, dirty });
                }
            }
        }
        if let Some(ws) = current_ws.take() {
            self.workspace_view.workspaces.push(ws);
        }
        self.workspace_view.ws_idx = 0;
        self.workspace_view.repo_idx = 0;
    }

    pub fn workspace_move_up(&mut self) {
        match self.workspace_view.focus {
            WorkspaceFocus::Workspaces => {
                if self.workspace_view.ws_idx > 0 { self.workspace_view.ws_idx -= 1; }
                self.workspace_view.repo_idx = 0;
            }
            WorkspaceFocus::Repos => {
                if self.workspace_view.repo_idx > 0 { self.workspace_view.repo_idx -= 1; }
            }
        }
    }

    pub fn workspace_move_down(&mut self) {
        match self.workspace_view.focus {
            WorkspaceFocus::Workspaces => {
                if self.workspace_view.ws_idx + 1 < self.workspace_view.workspaces.len() {
                    self.workspace_view.ws_idx += 1;
                }
                self.workspace_view.repo_idx = 0;
            }
            WorkspaceFocus::Repos => {
                let repo_len = self.workspace_view.workspaces
                    .get(self.workspace_view.ws_idx)
                    .map(|ws| ws.repos.len())
                    .unwrap_or(0);
                if self.workspace_view.repo_idx + 1 < repo_len {
                    self.workspace_view.repo_idx += 1;
                }
            }
        }
    }

    pub fn workspace_focus_repos(&mut self) {
        self.workspace_view.focus = WorkspaceFocus::Repos;
        self.workspace_view.repo_idx = 0;
    }

    pub fn workspace_focus_workspaces(&mut self) {
        self.workspace_view.focus = WorkspaceFocus::Workspaces;
    }

    // ── Config helpers ───────────────────────────────────────────────────────

    fn load_config(&mut self) {
        self.config_view.entries.clear();
        let scope_flag = if self.config_view.scope == ConfigScope::Local { "--local" } else { "--global" };
        let out = std::process::Command::new("torii")
            .args(["config", "list", scope_flag])
            .output();
        let Ok(out) = out else { return };
        let text = String::from_utf8_lossy(&out.stdout);
        let mut current_section = String::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('⚙') || line.starts_with("Global") || line.starts_with("Local") { continue; }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                let section = key.split('.').next().unwrap_or("").to_string();
                if section != current_section { current_section = section.clone(); }
                self.config_view.entries.push(ConfigEntry {
                    key,
                    value,
                    scope: self.config_view.scope.clone(),
                    section,
                });
            }
        }
        self.config_view.idx = 0;
    }

    pub fn config_move_up(&mut self) {
        if self.config_view.idx > 0 { self.config_view.idx -= 1; }
    }

    pub fn config_move_down(&mut self) {
        if self.config_view.idx + 1 < self.config_view.entries.len() {
            self.config_view.idx += 1;
        }
    }

    pub fn config_start_edit(&mut self) {
        if let Some(entry) = self.config_view.entries.get(self.config_view.idx) {
            self.config_view.edit_buf = entry.value.clone();
            self.config_view.edit_cursor = entry.value.len();
            self.config_view.editing = true;
        }
    }

    pub fn config_type_char(&mut self, c: char) {
        let cur = self.config_view.edit_cursor;
        self.config_view.edit_buf.insert(cur, c);
        self.config_view.edit_cursor += 1;
    }

    pub fn config_backspace(&mut self) {
        let cur = self.config_view.edit_cursor;
        if cur > 0 {
            self.config_view.edit_buf.remove(cur - 1);
            self.config_view.edit_cursor -= 1;
        }
    }

    pub fn config_cursor_left(&mut self) {
        if self.config_view.edit_cursor > 0 { self.config_view.edit_cursor -= 1; }
    }

    pub fn config_cursor_right(&mut self) {
        let len = self.config_view.edit_buf.len();
        if self.config_view.edit_cursor < len { self.config_view.edit_cursor += 1; }
    }

    // ── Settings helpers ─────────────────────────────────────────────────────

    pub fn settings_move_up(&mut self) {
        if self.settings_view.idx > 0 { self.settings_view.idx -= 1; }
    }

    pub fn settings_move_down(&mut self) {
        if self.settings_view.idx < 19 { self.settings_view.idx += 1; }
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

fn repo_quick_status(path: &str) -> (String, usize, usize, bool) {
    let Ok(repo) = Repository::discover(path) else { return ("?".into(), 0, 0, false) };
    let branch = repo.head().ok()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "detached".to_string());
    let (ahead, behind) = ahead_behind(&repo, &branch).unwrap_or((0, 0));
    let dirty = repo.statuses(None)
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    (branch, ahead, behind, dirty)
}
