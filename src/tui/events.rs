use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;
use crate::error::Result;
use super::app::{App, View, Panel, SyncStatus, CommitFocus};


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

        (_, KeyCode::Tab) => { app.sidebar_focused = true; return None; }

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
    match (key.modifiers, key.code) {
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.workspace_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.workspace_move_down(),
        (_, KeyCode::Enter)                          => return Some(Action::WorkspaceSync),
        _ => {}
    }
    None
}
