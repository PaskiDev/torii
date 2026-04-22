use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;
use crate::error::Result;
use super::app::{App, View, Panel, SyncStatus, CommitFocus, WorkspaceFocus, SnapshotFocus, BranchConfirm, TagConfirm, HistoryConfirm};


pub enum Action {
    Quit,
    Refresh,
    SidebarUp,
    SidebarDown,
    SidebarEnter,
    StageFile,
    UnstageFile,
    CommitConfirm,
    BranchCheckout,
    BranchDelete,
    BranchCreate,
    BranchPush,
    SnapshotRestore,
    SnapshotCreate,
    SnapshotDelete,
    SnapshotSaveInterval,
    OpenDiffFromLog,
    LogCopyHash,
    SyncRun,
    TagPush,
    TagDelete,
    TagCreate,
    HistoryCherryPick,
    HistoryRebase,
    HistoryScan,
    HistoryClean,
    HistoryRemoveFile,
    HistoryRewrite,
    HistoryBlame,
    RemoteInfo,
    MirrorSync,
    WorkspaceSync,
    WorkspaceSyncOne,
    WorkspaceOpenRepo,
    ConfigEdit,
    ConfigSave,
    ConfigToggleScope,
    SettingsToggle,
    SettingsSave,
    SettingsEditKeybind,
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self { Self }

    pub fn next(&mut self, app: &mut App) -> Result<Option<Action>> {
        if !event::poll(Duration::from_millis(200))? {
            return Ok(None);
        }

        match event::read()? {
            Event::Key(key) => {
                // Clear event log when panel is open
                if app.show_event_log {
                    if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::NONE {
                        app.event_log.clear();
                        return Ok(None);
                    }
                }
                // Tab cycles focus: sidebar → view panels → sidebar
                if key.code == KeyCode::Tab && key.modifiers == KeyModifiers::NONE {
                    app.tab_cycle();
                    return Ok(None);
                }
                // e toggles event log from anywhere — except when typing in a text input
                let typing = match app.view {
                    View::Commit    => app.commit_view.focus == CommitFocus::Input,
                    View::Snapshot  => app.snapshot_view.focus == SnapshotFocus::Create,
                    View::Log       => app.log.search_mode,
                    View::Branch    => app.branch_view.confirm == BranchConfirm::NewBranch,
                    View::Tag       => matches!(app.tag_view.confirm, TagConfirm::CreateName | TagConfirm::CreateMessage),
                    View::History   => matches!(app.history_view.confirm,
                        HistoryConfirm::Rebase | HistoryConfirm::RemoveFile |
                        HistoryConfirm::RewriteStart | HistoryConfirm::RewriteEnd |
                        HistoryConfirm::Blame),
                    _               => false,
                };
                if key.code == KeyCode::Char('e') && key.modifiers == KeyModifiers::NONE && !typing {
                    app.show_event_log = !app.show_event_log;
                    return Ok(None);
                }
                // Sidebar navigation takes priority when focused
                // but pass Enter and action keys to the current view too
                if app.sidebar_focused {
                    let is_nav = matches!(key.code,
                        KeyCode::Up | KeyCode::Down |
                        KeyCode::Char('j') | KeyCode::Char('k')
                    ) && key.modifiers == KeyModifiers::NONE;
                    let is_enter = key.code == KeyCode::Enter && key.modifiers == KeyModifiers::NONE;
                    let is_quit = matches!(key.code, KeyCode::Char('q'))
                        || (key.modifiers == KeyModifiers::CONTROL && key.code == KeyCode::Char('c'));
                    if is_nav || is_enter || is_quit {
                        return Ok(handle_sidebar(key, app));
                    }
                    // For action keys, delegate to view handler
                    let view_result = match app.view {
                        View::Log       => handle_log(key, app),
                        View::Branch    => handle_branch(key, app),
                        View::Tag       => handle_tag(key, app),
                        View::History   => handle_history(key, app),
                        View::Remote    => handle_remote(key, app),
                        View::Mirror    => handle_mirror(key, app),
                        View::Snapshot  => handle_snapshot(key, app),
                        View::Workspace => handle_workspace(key, app),
                        _ => None,
                    };
                    if view_result.is_some() {
                        return Ok(view_result);
                    }
                    return Ok(handle_sidebar(key, app));
                }
                // Esc always returns focus to sidebar unless the view handles it specially
                if key.code == KeyCode::Esc && key.modifiers == KeyModifiers::NONE {
                    let handled_by_view = match app.view {
                        View::Diff      => { app.go_back(); true }
                        View::Commit    => app.commit_view.focus == CommitFocus::Input,
                        View::Config    => app.config_view.editing,
                        View::Settings  => app.settings_view.editing_keybind.is_some(),
                        View::Log       => {
                            if app.log.ops_mode {
                                app.log.ops_mode = false;
                                true
                            } else if app.log.search_mode {
                                app.log.search_mode = false;
                                app.log.search_query.clear();
                                app.log.filtered.clear();
                                true
                            } else {
                                false
                            }
                        }
                        View::Branch    => {
                            if app.branch_view.ops_mode {
                                app.branch_view.ops_mode = false;
                                true
                            } else if app.branch_view.confirm != BranchConfirm::None {
                                app.branch_view.confirm = BranchConfirm::None;
                                app.branch_view.new_name.clear();
                                true
                            } else {
                                false
                            }
                        }
                        View::Tag       => {
                            if app.tag_view.ops_mode {
                                app.tag_view.ops_mode = false;
                                true
                            } else if app.tag_view.confirm != TagConfirm::None {
                                app.tag_view.confirm = TagConfirm::None;
                                app.tag_view.new_name.clear();
                                app.tag_view.new_message.clear();
                                true
                            } else {
                                false
                            }
                        }
                        View::History   => {
                            if app.history_view.ops_mode {
                                app.history_view.ops_mode = false;
                                true
                            } else if app.history_view.confirm != HistoryConfirm::None {
                                app.history_view.confirm = HistoryConfirm::None;
                                app.history_view.input.clear();
                                app.history_view.input2.clear();
                                true
                            } else {
                                false
                            }
                        }
                        View::Snapshot  => {
                            if app.snapshot_view.ops_mode {
                                app.snapshot_view.ops_mode = false;
                                true
                            } else {
                                false
                            }
                        }
                        _               => false,
                    };
                    if !handled_by_view {
                        app.sidebar_focused = true;
                    }
                    return Ok(None);
                }
                return Ok(match app.view {
                    View::Dashboard => handle_dashboard(key, app),
                    View::Diff      => handle_diff(key, app),
                    View::Log       => handle_log(key, app),
                    View::Branch    => handle_branch(key, app),
                    View::Commit    => handle_commit(key, app),
                    View::Snapshot  => handle_snapshot(key, app),
                    View::Sync      => handle_sync(key, app),
                    View::Tag       => handle_tag(key, app),
                    View::History   => handle_history(key, app),
                    View::Remote    => handle_remote(key, app),
                    View::Mirror    => handle_mirror(key, app),
                    View::Workspace => handle_workspace(key, app),
                    View::Config    => handle_config(key, app),
                    View::Settings  => handle_settings(key, app),
                    View::Help      => handle_help(key, app),
                });
            }
            Event::Resize(_, _) => {}
            _ => {}
        }

        Ok(None)
    }
}

fn handle_dashboard(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    // Global nav first
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }

    match (key.modifiers, key.code) {
        (_, KeyCode::BackTab) | (KeyModifiers::SHIFT, KeyCode::BackTab) => app.prev_panel(),
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.move_down(),

        (KeyModifiers::CONTROL, KeyCode::Char('r')) => return Some(Action::Refresh),

        (_, KeyCode::Char(' ')) => {
            match app.dashboard.selected_panel {
                Panel::Unstaged | Panel::Untracked => return Some(Action::StageFile),
                Panel::Staged                      => return Some(Action::UnstageFile),
                Panel::Log                         => {}
            }
        }

        (_, KeyCode::Char('d')) => app.go_to(View::Diff),

        _ => {}
    }
    None
}

fn handle_global_nav(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q')) |
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),

        (_, KeyCode::Char('?')) => app.go_to(View::Help),

        (_, KeyCode::Char('f')) => app.go_to(View::Dashboard),

        (_, KeyCode::Char('c')) => {
            app.commit_view.message.clear();
            app.commit_view.cursor = 0;
            app.commit_view.focus = CommitFocus::List;
            app.go_to(View::Commit);
        }
        (_, KeyCode::Char('s')) => {
            app.sync_view.status = SyncStatus::Idle;
            app.go_to(View::Sync);
        }
        (_, KeyCode::Char('p')) => app.go_to(View::Snapshot),
        (_, KeyCode::Char('l')) => app.go_to(View::Log),
        (_, KeyCode::Char('b')) => app.go_to(View::Branch),
        (_, KeyCode::Char('t')) => app.go_to(View::Tag),
        (_, KeyCode::Char('h')) => app.go_to(View::History),
        (_, KeyCode::Char('r')) => app.go_to(View::Remote),
        (_, KeyCode::Char('m')) => app.go_to(View::Mirror),
        (_, KeyCode::Char('w')) => app.go_to(View::Workspace),
        (_, KeyCode::Char('g')) => app.go_to(View::Config),
        (_, KeyCode::Char('x')) => app.go_to(View::Settings),
        _ => return None,
    }
    None
}

fn handle_diff(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => app.go_back(),
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.diff_scroll_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.diff_scroll_down(),
        (_, KeyCode::PageUp)                          => app.diff_page_up(),
        (_, KeyCode::PageDown)                        => app.diff_page_down(),
        (KeyModifiers::CONTROL, KeyCode::Char('c'))  => return Some(Action::Quit),
        _ => {}
    }
    None
}

fn handle_log(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if app.log.search_mode {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => {
                app.log.search_mode = false;
                app.log.search_query.clear();
                app.log.filtered.clear();
            }
            (_, KeyCode::Enter) => {
                app.log.search_mode = false;
            }
            (_, KeyCode::Backspace) => {
                app.log.search_query.pop();
                app.log_update_filter();
            }
            (_, KeyCode::Char(c)) if key.modifiers == KeyModifiers::NONE ||
                                      key.modifiers == KeyModifiers::SHIFT
                                    => {
                app.log.search_query.push(c);
                app.log_update_filter();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
            _ => {}
        }
        return None;
    }
    if app.log.ops_mode {
        match (key.modifiers, key.code) {
            (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                if app.log.ops_idx > 0 { app.log.ops_idx -= 1; }
            }
            (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                if app.log.ops_idx < 2 { app.log.ops_idx += 1; }
            }
            (_, KeyCode::Enter) => {
                let idx = app.log.ops_idx;
                app.log.ops_mode = false;
                match idx {
                    0 => return Some(Action::OpenDiffFromLog),
                    1 => return Some(Action::LogCopyHash),
                    2 => {
                        app.log.search_mode = true;
                        app.log.search_query.clear();
                        app.log.filtered.clear();
                    }
                    _ => {}
                }
            }
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                app.log.ops_mode = false;
            }
            _ => {}
        }
        return None;
    }
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)     | (_, KeyCode::Char('k')) => { app.log_move_up(); app.log.ops_mode = false; }
        (_, KeyCode::Down)   | (_, KeyCode::Char('j')) => { app.log_move_down(); app.log.ops_mode = false; }
        (_, KeyCode::Char('o')) => {
            app.log.ops_mode = true;
            app.log.ops_idx = 0;
        }
        (_, KeyCode::Char('/')) => {
            app.log.search_mode = true;
            app.log.search_query.clear();
            app.log.filtered.clear();
        }
        _ => {}
    }
    None
}

fn handle_branch(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match app.branch_view.confirm.clone() {
        BranchConfirm::Delete => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Char('y')) => {
                    app.branch_view.confirm = BranchConfirm::None;
                    return Some(Action::BranchDelete);
                }
                _ => {
                    app.branch_view.confirm = BranchConfirm::None;
                    app.branch_view.status = Some("cancelled".to_string());
                }
            }
            return None;
        }
        BranchConfirm::NewBranch => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Esc) => {
                    app.branch_view.confirm = BranchConfirm::None;
                    app.branch_view.new_name.clear();
                }
                (_, KeyCode::Enter) => {
                    app.branch_view.confirm = BranchConfirm::None;
                    return Some(Action::BranchCreate);
                }
                (_, KeyCode::Backspace) => { app.branch_view.new_name.pop(); }
                (_, KeyCode::Char(c)) if key.modifiers == KeyModifiers::NONE ||
                                          key.modifiers == KeyModifiers::SHIFT
                                        => app.branch_view.new_name.push(c),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
            return None;
        }
        BranchConfirm::None => {}
    }
    if app.branch_view.ops_mode {
        match (key.modifiers, key.code) {
            (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                if app.branch_view.ops_idx > 0 { app.branch_view.ops_idx -= 1; }
            }
            (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                if app.branch_view.ops_idx < 3 { app.branch_view.ops_idx += 1; }
            }
            (_, KeyCode::Enter) => {
                let idx = app.branch_view.ops_idx;
                app.branch_view.ops_mode = false;
                match idx {
                    0 => return Some(Action::BranchCheckout),
                    1 => {
                        app.branch_view.new_name.clear();
                        app.branch_view.confirm = BranchConfirm::NewBranch;
                    }
                    2 => {
                        if !app.branch_view.current_has_upstream {
                            return Some(Action::BranchPush);
                        }
                    }
                    3 => {
                        if let Some(b) = app.branch_view.branches.get(app.branch_view.idx) {
                            if !b.is_current && !b.is_remote {
                                app.branch_view.confirm = BranchConfirm::Delete;
                            } else if b.is_remote {
                                app.branch_view.status = Some("cannot delete remote branch".to_string());
                            } else {
                                app.branch_view.status = Some("cannot delete current branch".to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                app.branch_view.ops_mode = false;
            }
            _ => {}
        }
        return None;
    }
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)    | (_, KeyCode::Char('k')) => app.branch_move_up(),
        (_, KeyCode::Down)  | (_, KeyCode::Char('j')) => app.branch_move_down(),
        (_, KeyCode::Char('o')) => {
            app.branch_view.ops_mode = true;
            app.branch_view.ops_idx = 0;
        }
        _ => {}
    }
    None
}

fn handle_commit(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    const N_TYPES: usize = 8;
    match app.commit_view.focus {
        CommitFocus::List => {
            if let Some(a) = handle_global_nav(key, app) { return Some(a); }
            match (key.modifiers, key.code) {
                (_, KeyCode::Enter) => app.commit_view.focus = CommitFocus::TypeSelector,
                (_, KeyCode::Char('i')) => {
                    app.commit_view.focus = CommitFocus::Input;
                }
                _ => {}
            }
        }
        CommitFocus::TypeSelector => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => {
                    if app.commit_view.type_idx > 0 { app.commit_view.type_idx -= 1; }
                }
                (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                    if app.commit_view.type_idx < N_TYPES - 1 { app.commit_view.type_idx += 1; }
                }
                (_, KeyCode::Enter) => {
                    let prefix = COMMIT_TYPES[app.commit_view.type_idx].0;
                    let prefix_str = format!("{}: ", prefix);
                    if !app.commit_view.message.starts_with(&prefix_str) {
                        // Strip any existing type prefix first
                        let base = if let Some(colon) = app.commit_view.message.find(": ") {
                            app.commit_view.message[colon + 2..].to_string()
                        } else {
                            app.commit_view.message.clone()
                        };
                        app.commit_view.message = format!("{}{}", prefix_str, base);
                        app.commit_view.cursor = app.commit_view.message.len();
                    }
                    app.commit_view.focus = CommitFocus::Input;
                }
                (_, KeyCode::Esc) => app.commit_view.focus = CommitFocus::List,
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
        }
        CommitFocus::Input => match (key.modifiers, key.code) {
            (_, KeyCode::Esc)                            => app.commit_view.focus = CommitFocus::TypeSelector,
            (_, KeyCode::Enter)                          => return Some(Action::CommitConfirm),
            (_, KeyCode::Backspace)                      => app.commit_backspace(),
            (_, KeyCode::Left)                           => app.commit_cursor_left(),
            (_, KeyCode::Right)                          => app.commit_cursor_right(),
            (_, KeyCode::Char(c)) if key.modifiers == KeyModifiers::NONE ||
                                      key.modifiers == KeyModifiers::SHIFT
                                                         => app.commit_type_char(c),
            (KeyModifiers::CONTROL, KeyCode::Char('c'))  => return Some(Action::Quit),
            _ => {}
        },
    }
    None
}

pub const COMMIT_TYPES: &[(&str, &str)] = &[
    ("feat",     "new feature"),
    ("fix",      "bug fix"),
    ("chore",    "maintenance task"),
    ("docs",     "documentation"),
    ("refactor", "code restructure"),
    ("test",     "tests"),
    ("ci",       "CI/CD changes"),
    ("perf",     "performance improvement"),
];

fn handle_snapshot(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    use crate::tui::app::{SnapshotFocus, AutoSnapshotInterval};

    // ops dropdown (only in List focus)
    if app.snapshot_view.focus == SnapshotFocus::List && app.snapshot_view.ops_mode {
        const OPS_LEN: usize = 3;
        match (key.modifiers, key.code) {
            (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                if app.snapshot_view.ops_idx > 0 { app.snapshot_view.ops_idx -= 1; }
            }
            (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                if app.snapshot_view.ops_idx < OPS_LEN - 1 { app.snapshot_view.ops_idx += 1; }
            }
            (_, KeyCode::Enter) => {
                let idx = app.snapshot_view.ops_idx;
                app.snapshot_view.ops_mode = false;
                return match idx {
                    0 => Some(Action::SnapshotRestore),
                    1 => {
                        app.snapshot_view.create_name.clear();
                        app.snapshot_view.focus = SnapshotFocus::Create;
                        None
                    }
                    2 => Some(Action::SnapshotDelete),
                    _ => None,
                };
            }
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                app.snapshot_view.ops_mode = false;
            }
            _ => {}
        }
        return None;
    }

    match app.snapshot_view.focus {
        SnapshotFocus::List => {
            if let Some(a) = handle_global_nav(key, app) { return Some(a); }
            match (key.modifiers, key.code) {
                (_, KeyCode::Up)    | (_, KeyCode::Char('k')) => app.snapshot_move_up(),
                (_, KeyCode::Down)  | (_, KeyCode::Char('j')) => app.snapshot_move_down(),
                (_, KeyCode::Char('o')) => {
                    app.snapshot_view.ops_mode = true;
                    app.snapshot_view.ops_idx = 0;
                }
                (_, KeyCode::Char('n'))                       => {
                    app.snapshot_view.create_name.clear();
                    app.snapshot_view.focus = SnapshotFocus::Create;
                }
                (_, KeyCode::Char('a'))                       => {
                    app.snapshot_view.auto_interval_idx = AutoSnapshotInterval::all()
                        .iter().position(|i| i == &app.snapshot_view.auto_interval)
                        .unwrap_or(0);
                    app.snapshot_view.focus = SnapshotFocus::AutoConfig;
                }
                _ => {}
            }
        }
        SnapshotFocus::Create => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Esc)       => app.snapshot_view.focus = SnapshotFocus::List,
                (_, KeyCode::Enter)     => return Some(Action::SnapshotCreate),
                (_, KeyCode::Backspace) => { app.snapshot_view.create_name.pop(); }
                (_, KeyCode::Char(c)) if key.modifiers == KeyModifiers::NONE ||
                                          key.modifiers == KeyModifiers::SHIFT
                                        => app.snapshot_view.create_name.push(c),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
        }
        SnapshotFocus::AutoConfig => {
            let n = AutoSnapshotInterval::all().len();
            match (key.modifiers, key.code) {
                (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => {
                    if app.snapshot_view.auto_interval_idx > 0 {
                        app.snapshot_view.auto_interval_idx -= 1;
                    }
                }
                (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                    if app.snapshot_view.auto_interval_idx < n - 1 {
                        app.snapshot_view.auto_interval_idx += 1;
                    }
                }
                (_, KeyCode::Enter) => {
                    app.snapshot_view.auto_interval =
                        AutoSnapshotInterval::all()[app.snapshot_view.auto_interval_idx].clone();
                    app.snapshot_view.focus = SnapshotFocus::List;
                    return Some(Action::SnapshotSaveInterval);
                }
                (_, KeyCode::Esc) => app.snapshot_view.focus = SnapshotFocus::List,
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
        }
    }
    None
}

fn handle_sync(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.sync_op_prev(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.sync_op_next(),
        (_, KeyCode::Enter)                          => return Some(Action::SyncRun),
        _ => {}
    }
    None
}

fn handle_sidebar(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)    | (_, KeyCode::Char('k')) => return Some(Action::SidebarUp),
        (_, KeyCode::Down)  | (_, KeyCode::Char('j')) => return Some(Action::SidebarDown),
        (_, KeyCode::Enter)                           => return Some(Action::SidebarEnter),
        (_, KeyCode::Char('?'))                       => { app.go_to(View::Help); }
        (_, KeyCode::Char('q')) |
        (KeyModifiers::CONTROL, KeyCode::Char('c'))   => return Some(Action::Quit),
        _ => {}
    }
    None
}

fn handle_help(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (_, KeyCode::Char('?')) | (_, KeyCode::Char('q')) => app.go_back(),
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
        _ => {}
    }
    None
}

fn handle_tag(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match app.tag_view.confirm.clone() {
        TagConfirm::Delete => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Char('y')) => {
                    app.tag_view.confirm = TagConfirm::None;
                    return Some(Action::TagDelete);
                }
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => { app.tag_view.confirm = TagConfirm::None; }
            }
            return None;
        }
        TagConfirm::CreateName => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Enter) => {
                    if !app.tag_view.new_name.trim().is_empty() {
                        app.tag_view.confirm = TagConfirm::CreateMessage;
                    }
                }
                (_, KeyCode::Backspace) => { app.tag_view.new_name.pop(); }
                (KeyModifiers::NONE, KeyCode::Char(c)) => app.tag_view.new_name.push(c),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
            return None;
        }
        TagConfirm::CreateMessage => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Enter) => {
                    app.tag_view.confirm = TagConfirm::None;
                    return Some(Action::TagCreate);
                }
                (_, KeyCode::Backspace) => { app.tag_view.new_message.pop(); }
                (KeyModifiers::NONE, KeyCode::Char(c)) => app.tag_view.new_message.push(c),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
            return None;
        }
        TagConfirm::None => {}
    }
    if app.tag_view.ops_mode {
        match (key.modifiers, key.code) {
            (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                if app.tag_view.ops_idx > 0 { app.tag_view.ops_idx -= 1; }
            }
            (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                if app.tag_view.ops_idx < 2 { app.tag_view.ops_idx += 1; }
            }
            (_, KeyCode::Enter) => {
                let idx = app.tag_view.ops_idx;
                app.tag_view.ops_mode = false;
                match idx {
                    0 => return Some(Action::TagPush),
                    1 => {
                        app.tag_view.new_name.clear();
                        app.tag_view.new_message.clear();
                        app.tag_view.confirm = TagConfirm::CreateName;
                    }
                    2 => {
                        if !app.tag_view.tags.is_empty() {
                            app.tag_view.confirm = TagConfirm::Delete;
                        }
                    }
                    _ => {}
                }
            }
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                app.tag_view.ops_mode = false;
            }
            _ => {}
        }
        return None;
    }
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.tag_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.tag_move_down(),
        (_, KeyCode::Char('o')) => {
            app.tag_view.ops_mode = true;
            app.tag_view.ops_idx = 0;
        }
        _ => {}
    }
    None
}

fn handle_history(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match app.history_view.confirm.clone() {
        HistoryConfirm::CherryPick => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Char('y')) => {
                    app.history_view.confirm = HistoryConfirm::None;
                    return Some(Action::HistoryCherryPick);
                }
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => { app.history_view.confirm = HistoryConfirm::None; }
            }
            return None;
        }
        HistoryConfirm::Clean => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Char('y')) => {
                    app.history_view.confirm = HistoryConfirm::None;
                    return Some(Action::HistoryClean);
                }
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => { app.history_view.confirm = HistoryConfirm::None; }
            }
            return None;
        }
        HistoryConfirm::RemoveFile => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Enter) => {
                    if !app.history_view.input.trim().is_empty() {
                        app.history_view.confirm = HistoryConfirm::None;
                        return Some(Action::HistoryRemoveFile);
                    }
                }
                (_, KeyCode::Backspace) => { app.history_view.input.pop(); }
                (KeyModifiers::NONE, KeyCode::Char(c)) => app.history_view.input.push(c),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
            return None;
        }
        HistoryConfirm::Rebase => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Enter) => {
                    if !app.history_view.input.trim().is_empty() {
                        app.history_view.confirm = HistoryConfirm::None;
                        return Some(Action::HistoryRebase);
                    }
                }
                (_, KeyCode::Backspace) => { app.history_view.input.pop(); }
                (KeyModifiers::NONE, KeyCode::Char(c)) => app.history_view.input.push(c),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
            return None;
        }
        HistoryConfirm::RewriteStart => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Enter) => {
                    if !app.history_view.input.trim().is_empty() {
                        app.history_view.confirm = HistoryConfirm::RewriteEnd;
                    }
                }
                (_, KeyCode::Backspace) => { app.history_view.input.pop(); }
                (KeyModifiers::NONE, KeyCode::Char(c)) => app.history_view.input.push(c),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
            return None;
        }
        HistoryConfirm::RewriteEnd => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Enter) => {
                    if !app.history_view.input2.trim().is_empty() {
                        app.history_view.confirm = HistoryConfirm::None;
                        return Some(Action::HistoryRewrite);
                    }
                }
                (_, KeyCode::Backspace) => { app.history_view.input2.pop(); }
                (KeyModifiers::NONE, KeyCode::Char(c)) => app.history_view.input2.push(c),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
            return None;
        }
        HistoryConfirm::Blame => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Enter) => {
                    if !app.history_view.input.trim().is_empty() {
                        app.history_view.confirm = HistoryConfirm::None;
                        return Some(Action::HistoryBlame);
                    }
                }
                (_, KeyCode::Backspace) => { app.history_view.input.pop(); }
                (KeyModifiers::NONE, KeyCode::Char(c)) => app.history_view.input.push(c),
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => {}
            }
            return None;
        }
        HistoryConfirm::Scan => {
            match (key.modifiers, key.code) {
                (_, KeyCode::Char('f')) => {
                    app.history_view.scan_full = !app.history_view.scan_full;
                }
                (_, KeyCode::Enter) => {
                    app.history_view.confirm = HistoryConfirm::None;
                    return Some(Action::HistoryScan);
                }
                (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
                _ => { app.history_view.confirm = HistoryConfirm::None; }
            }
            return None;
        }
        HistoryConfirm::None => {}
    }
    if app.history_view.ops_mode {
        match (key.modifiers, key.code) {
            (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                if app.history_view.ops_idx > 0 { app.history_view.ops_idx -= 1; }
            }
            (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                if app.history_view.ops_idx < 6 { app.history_view.ops_idx += 1; }
            }
            (_, KeyCode::Enter) => {
                let idx = app.history_view.ops_idx;
                app.history_view.ops_mode = false;
                match idx {
                    0 => { app.history_view.confirm = HistoryConfirm::CherryPick; }
                    1 => { app.history_view.input.clear(); app.history_view.confirm = HistoryConfirm::Rebase; }
                    2 => { app.history_view.confirm = HistoryConfirm::Scan; }
                    3 => { app.history_view.confirm = HistoryConfirm::Clean; }
                    4 => { app.history_view.input.clear(); app.history_view.confirm = HistoryConfirm::Blame; }
                    5 => { app.history_view.input.clear(); app.history_view.input2.clear(); app.history_view.confirm = HistoryConfirm::RewriteStart; }
                    6 => { app.history_view.input.clear(); app.history_view.confirm = HistoryConfirm::RemoveFile; }
                    _ => {}
                }
            }
            (_, KeyCode::Esc) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                app.history_view.ops_mode = false;
            }
            _ => {}
        }
        return None;
    }
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.history_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.history_move_down(),
        (_, KeyCode::Char('o')) => {
            app.history_view.ops_mode = true;
            app.history_view.ops_idx = 0;
        }
        _ => {}
    }
    None
}

fn handle_remote(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.remote_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.remote_move_down(),
        (_, KeyCode::Enter)                          => return Some(Action::RemoteInfo),
        _ => {}
    }
    None
}

fn handle_mirror(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.mirror_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.mirror_move_down(),
        (_, KeyCode::Enter)                          => return Some(Action::MirrorSync),
        _ => {}
    }
    None
}

fn handle_workspace(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match app.workspace_view.focus {
        WorkspaceFocus::Workspaces => match (key.modifiers, key.code) {
            (_, KeyCode::Up)    | (_, KeyCode::Char('k')) => app.workspace_move_up(),
            (_, KeyCode::Down)  | (_, KeyCode::Char('j')) => app.workspace_move_down(),
            (_, KeyCode::Right) | (_, KeyCode::Char('l')) => app.workspace_focus_repos(),
            (_, KeyCode::Enter)                           => return Some(Action::WorkspaceSync),
            _ => {}
        },
        WorkspaceFocus::Repos => match (key.modifiers, key.code) {
            (_, KeyCode::Up)    | (_, KeyCode::Char('k')) => app.workspace_move_up(),
            (_, KeyCode::Down)  | (_, KeyCode::Char('j')) => app.workspace_move_down(),
            (_, KeyCode::Left)  | (_, KeyCode::Char('h')) => app.workspace_focus_workspaces(),
            (_, KeyCode::Esc)                             => app.workspace_focus_workspaces(),
            (_, KeyCode::Enter)                           => return Some(Action::WorkspaceOpenRepo),
            (_, KeyCode::Char('s'))                       => return Some(Action::WorkspaceSyncOne),
            (_, KeyCode::Char('S'))                       => return Some(Action::WorkspaceSync),
            _ => {}
        },
    }
    None
}

fn handle_config(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if app.config_view.editing {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc)       => { app.config_view.editing = false; }
            (_, KeyCode::Enter)     => return Some(Action::ConfigSave),
            (_, KeyCode::Backspace) => app.config_backspace(),
            (_, KeyCode::Left)      => app.config_cursor_left(),
            (_, KeyCode::Right)     => app.config_cursor_right(),
            (_, KeyCode::Char(c)) if key.modifiers == KeyModifiers::NONE ||
                                      key.modifiers == KeyModifiers::SHIFT
                                    => app.config_type_char(c),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),
            _ => {}
        }
        return None;
    }
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.config_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.config_move_down(),
        (_, KeyCode::Enter)                          => { app.config_start_edit(); return Some(Action::ConfigEdit); }
        (_, KeyCode::Tab)                            => return Some(Action::ConfigToggleScope),
        _ => {}
    }
    None
}

fn handle_settings(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if let Some(editing_idx) = app.settings_view.editing_keybind {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc) => { app.settings_view.editing_keybind = None; }
            (_, KeyCode::Char(c)) if key.modifiers == KeyModifiers::NONE ||
                                      key.modifiers == KeyModifiers::SHIFT => {
                apply_keybind(app, editing_idx, c);
                app.settings_view.editing_keybind = None;
                app.settings_view.status = Some(format!("keybind updated"));
            }
            _ => {}
        }
        return None;
    }
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.settings_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.settings_move_down(),
        (_, KeyCode::Enter)                          => return Some(Action::SettingsToggle),
        (_, KeyCode::Char('s'))                      => return Some(Action::SettingsSave),
        _ => {}
    }
    None
}

fn apply_keybind(app: &mut App, idx: usize, c: char) {
    match idx {
        8  => app.settings.keybind_files = c,
        9  => app.settings.keybind_save = c,
        10 => app.settings.keybind_sync = c,
        11 => app.settings.keybind_snapshot = c,
        12 => app.settings.keybind_log = c,
        13 => app.settings.keybind_branch = c,
        14 => app.settings.keybind_tag = c,
        15 => app.settings.keybind_history = c,
        16 => app.settings.keybind_remote = c,
        17 => app.settings.keybind_mirror = c,
        18 => app.settings.keybind_workspace = c,
        19 => app.settings.keybind_config = c,
        _  => {}
    }
}
