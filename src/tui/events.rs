use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;
use crate::error::Result;
use super::app::{App, View, Panel};

pub enum Action {
    Quit,
    Refresh,
    StageFile,
    UnstageFile,
    CommitConfirm,
    BranchCheckout,
    SnapshotRestore,
    OpenDiffFromLog,
    SyncRun,
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
                return Ok(match app.view {
                    View::Dashboard => handle_dashboard(key, app),
                    View::Diff      => handle_diff(key, app),
                    View::Log       => handle_log(key, app),
                    View::Branch    => handle_branch(key, app),
                    View::Commit    => handle_commit(key, app),
                    View::Snapshot  => handle_snapshot(key, app),
                    View::Sync      => handle_sync(key, app),
                });
            }
            Event::Resize(_, _) => {}
            _ => {}
        }

        Ok(None)
    }
}

fn handle_dashboard(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Char('q')) |
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => return Some(Action::Quit),

        (_, KeyCode::Tab)                            => app.next_panel(),
        (KeyModifiers::SHIFT, KeyCode::BackTab)      => app.prev_panel(),
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.move_down(),

        (_, KeyCode::Char('r')) => return Some(Action::Refresh),

        (_, KeyCode::Char(' ')) => {
            match app.dashboard.selected_panel {
                Panel::Unstaged | Panel::Untracked => return Some(Action::StageFile),
                Panel::Staged                      => return Some(Action::UnstageFile),
                Panel::Log                         => {}
            }
        }

        (_, KeyCode::Char('d')) => app.go_to(View::Diff),
        (_, KeyCode::Char('l')) => app.go_to(View::Log),

        // v = vault (save/commit)
        (_, KeyCode::Char('v')) => {
            app.commit_view.message.clear();
            app.commit_view.cursor = 0;
            app.go_to(View::Commit);
        }

        // s = sync
        (_, KeyCode::Char('s')) => app.go_to(View::Sync),

        // x = snapshot
        (_, KeyCode::Char('x')) => app.go_to(View::Snapshot),

        (_, KeyCode::Char('b')) => app.go_to(View::Branch),

        _ => {}
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
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => app.go_back(),
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.log_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.log_move_down(),
        (_, KeyCode::Char('d')) => return Some(Action::OpenDiffFromLog),
        (KeyModifiers::CONTROL, KeyCode::Char('c'))  => return Some(Action::Quit),
        _ => {}
    }
    None
}

fn handle_branch(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => app.go_back(),
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.branch_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.branch_move_down(),
        (_, KeyCode::Enter)                          => return Some(Action::BranchCheckout),
        (KeyModifiers::CONTROL, KeyCode::Char('c'))  => return Some(Action::Quit),
        _ => {}
    }
    None
}

fn handle_commit(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc)                            => app.go_back(),
        (_, KeyCode::Enter)                          => return Some(Action::CommitConfirm),
        (_, KeyCode::Backspace)                      => app.commit_backspace(),
        (_, KeyCode::Left)                           => app.commit_cursor_left(),
        (_, KeyCode::Right)                          => app.commit_cursor_right(),
        (_, KeyCode::Char(c)) if key.modifiers == KeyModifiers::NONE ||
                                  key.modifiers == KeyModifiers::SHIFT
                                                     => app.commit_type_char(c),
        (KeyModifiers::CONTROL, KeyCode::Char('c'))  => return Some(Action::Quit),
        _ => {}
    }
    None
}

fn handle_snapshot(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => app.go_back(),
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.snapshot_move_up(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.snapshot_move_down(),
        (_, KeyCode::Enter)                          => return Some(Action::SnapshotRestore),
        (KeyModifiers::CONTROL, KeyCode::Char('c'))  => return Some(Action::Quit),
        _ => {}
    }
    None
}

fn handle_sync(key: event::KeyEvent, app: &mut App) -> Option<Action> {
    match (key.modifiers, key.code) {
        (_, KeyCode::Esc) | (_, KeyCode::Char('q')) => app.go_back(),
        (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.sync_op_prev(),
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.sync_op_next(),
        (_, KeyCode::Enter)                          => return Some(Action::SyncRun),
        (KeyModifiers::CONTROL, KeyCode::Char('c'))  => return Some(Action::Quit),
        _ => {}
    }
    None
}
