use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;
use crate::error::Result;
use super::app::{App, View, Panel, SyncStatus, CommitFocus, WorkspaceFocus};


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
    SnapshotRestore,
    OpenDiffFromLog,
    SyncRun,
    TagPush,
    TagDelete,
    HistoryCherryPick,
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
                // e toggles event log from anywhere
                if key.code == KeyCode::Char('e') && key.modifiers == KeyModifiers::NONE {
                    app.show_event_log = !app.show_event_log;
                    return Ok(None);
                }
                // Sidebar navigation takes priority when focused
                if app.sidebar_focused {
                    return Ok(handle_sidebar(key, app));
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
        (KeyModifiers::CONTROL, KeyCode::Char('c'))  => return Some(Action::Quit),
        _ => {}
    }
    None
}

fn handle_log(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.log_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.log_move_down(),
        (_, KeyCode::Char('d')) => return Some(Action::OpenDiffFromLog),
        _ => {}
    }
    None
}

fn handle_branch(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.branch_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.branch_move_down(),
        (_, KeyCode::Enter)                          => return Some(Action::BranchCheckout),
        _ => {}
    }
    None
}

fn handle_commit(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match app.commit_view.focus {
        CommitFocus::List => {
            if let Some(a) = handle_global_nav(key, app) { return Some(a); }
            match (key.modifiers, key.code) {
                (_, KeyCode::Enter) => app.commit_view.focus = CommitFocus::Input,
                (_, KeyCode::Esc)                       => app.go_back(),
                _ => {}
            }
        }
        CommitFocus::Input => match (key.modifiers, key.code) {
            (_, KeyCode::Esc)                            => app.commit_view.focus = CommitFocus::List,
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

fn handle_snapshot(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.snapshot_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.snapshot_move_down(),
        (_, KeyCode::Enter)                          => return Some(Action::SnapshotRestore),
        _ => {}
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
        (_, KeyCode::Esc)                             => { app.sidebar_focused = false; }
        // Tab handled globally before reaching here — this branch unreachable but harmless
        (_, KeyCode::Char('?'))                       => { app.sidebar_focused = false; app.go_to(View::Help); }
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
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.tag_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.tag_move_down(),
        (_, KeyCode::Enter)                          => return Some(Action::TagPush),
        (_, KeyCode::Char('d'))                      => return Some(Action::TagDelete),
        _ => {}
    }
    None
}

fn handle_history(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    if let Some(a) = handle_global_nav(key, app) { return Some(a); }
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.history_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.history_move_down(),
        (_, KeyCode::Enter)                          => return Some(Action::HistoryCherryPick),
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
