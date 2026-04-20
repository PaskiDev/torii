pub mod app;
pub mod events;
pub mod ui;
pub mod views;

use std::io;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, View, SyncOp, SyncStatus};
use events::{Action, EventHandler};

pub fn run() -> crate::error::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let mut events = EventHandler::new();

    let result = run_loop(&mut terminal, &mut app, &mut events);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &mut EventHandler,
) -> crate::error::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Some(action) = events.next(app)? {
            match action {
                Action::Quit => break,

                Action::Refresh => {
                    app.refresh()?;
                    app.set_status("refreshed");
                }

                Action::StageFile => {
                    stage_selected(app)?;
                    app.refresh()?;
                }

                Action::UnstageFile => {
                    unstage_selected(app)?;
                    app.refresh()?;
                }

                Action::CommitConfirm => {
                    let msg = app.commit_view.message.trim().to_string();
                    if !msg.is_empty() && !app.staged.is_empty() {
                        commit_staged(app, &msg)?;
                        app.commit_view.message.clear();
                        app.commit_view.cursor = 0;
                        app.go_back();
                        app.refresh()?;
                        app.set_status(format!("committed: {}", truncate_msg(&msg, 40)));
                    }
                }

                Action::BranchCheckout => {
                    checkout_selected(app)?;
                    app.go_back();
                    app.refresh()?;
                }

                Action::SnapshotRestore => {
                    restore_selected_snapshot(app)?;
                    app.go_back();
                    app.refresh()?;
                }

                Action::OpenDiffFromLog => {
                    app.dashboard.log_idx = app.log.idx;
                    app.dashboard.selected_panel = app::Panel::Log;
                    app.go_to(View::Diff);
                }

                Action::SyncRun => {
                    run_sync(app);
                    app.refresh()?;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

// ── Git operations ────────────────────────────────────────────────────────────

fn stage_selected(app: &mut App) -> crate::error::Result<()> {
    use git2::Repository;
    use app::Panel;

    let repo = Repository::discover(&app.repo_path).map_err(crate::error::ToriiError::Git)?;
    let mut index = repo.index().map_err(crate::error::ToriiError::Git)?;

    let path = match app.dashboard.selected_panel {
        Panel::Unstaged  => app.unstaged.get(app.dashboard.unstaged_idx).map(|e| e.path.clone()),
        Panel::Untracked => app.untracked.get(app.dashboard.untracked_idx).map(|e| e.path.clone()),
        _ => None,
    };

    if let Some(p) = path {
        index.add_path(std::path::Path::new(&p)).map_err(crate::error::ToriiError::Git)?;
        index.write().map_err(crate::error::ToriiError::Git)?;
        app.set_status(format!("staged: {}", p));
    }
    Ok(())
}

fn unstage_selected(app: &mut App) -> crate::error::Result<()> {
    use git2::Repository;

    let repo = Repository::discover(&app.repo_path).map_err(crate::error::ToriiError::Git)?;
    let path = app.staged.get(app.dashboard.staged_idx).map(|e| e.path.clone());

    if let Some(p) = path {
        let head = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        if let Some(commit) = head {
            let treeish = commit.as_object();
            repo.reset_default(Some(treeish), [p.as_str()].iter())
                .map_err(crate::error::ToriiError::Git)?;
        } else {
            // No commits yet — remove from index directly
            let mut index = repo.index().map_err(crate::error::ToriiError::Git)?;
            index.remove_path(std::path::Path::new(&p)).map_err(crate::error::ToriiError::Git)?;
            index.write().map_err(crate::error::ToriiError::Git)?;
        }
        app.set_status(format!("unstaged: {}", p));
    }
    Ok(())
}

fn commit_staged(app: &App, message: &str) -> crate::error::Result<()> {
    use git2::{Repository, Signature};

    let repo = Repository::discover(&app.repo_path).map_err(crate::error::ToriiError::Git)?;
    let mut index = repo.index().map_err(crate::error::ToriiError::Git)?;
    let tree_oid = index.write_tree().map_err(crate::error::ToriiError::Git)?;
    let tree = repo.find_tree(tree_oid).map_err(crate::error::ToriiError::Git)?;

    let sig = repo.signature().map_err(crate::error::ToriiError::Git)?;
    let author = Signature::now(sig.name().unwrap_or(""), sig.email().unwrap_or(""))
        .map_err(crate::error::ToriiError::Git)?;

    let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
    let parents: Vec<&git2::Commit> = parent.iter().collect();

    repo.commit(Some("HEAD"), &author, &author, message, &tree, &parents)
        .map_err(crate::error::ToriiError::Git)?;

    Ok(())
}

fn checkout_selected(app: &mut App) -> crate::error::Result<()> {
    use git2::Repository;

    let repo = Repository::discover(&app.repo_path).map_err(crate::error::ToriiError::Git)?;
    let branch = app.branch_view.branches.get(app.branch_view.idx);

    if let Some(b) = branch {
        if b.is_remote {
            app.set_status(format!("cannot checkout remote directly: {}", b.name));
            return Ok(());
        }
        let obj = repo.revparse_single(&format!("refs/heads/{}", b.name))
            .map_err(crate::error::ToriiError::Git)?;
        repo.checkout_tree(&obj, None).map_err(crate::error::ToriiError::Git)?;
        repo.set_head(&format!("refs/heads/{}", b.name))
            .map_err(crate::error::ToriiError::Git)?;
        app.set_status(format!("switched to: {}", b.name));
    }
    Ok(())
}

fn restore_selected_snapshot(app: &mut App) -> crate::error::Result<()> {
    // Delegate to torii snapshot restore logic via subprocess
    let snap = app.snapshot_view.snapshots.get(app.snapshot_view.idx);
    if let Some(s) = snap {
        let id = s.id.clone();
        app.set_status(format!("restoring snapshot: {}", id));
        // Actual restore would call crate::snapshot::restore(&id)
        // For now: placeholder — wired in next iteration
    }
    Ok(())
}

fn run_sync(app: &mut App) {
    use crate::core::GitRepo;

    app.sync_view.status = SyncStatus::Running;

    let result = match app.sync_view.selected_op {
        SyncOp::PullPush => {
            GitRepo::open(&app.repo_path)
                .and_then(|r| r.pull().and_then(|_| r.push(false)))
                .map(|_| "synced with remote".to_string())
        }
        SyncOp::PullOnly => {
            GitRepo::open(&app.repo_path)
                .and_then(|r| r.pull())
                .map(|_| "pulled from remote".to_string())
        }
        SyncOp::PushOnly => {
            GitRepo::open(&app.repo_path)
                .and_then(|r| r.push(false))
                .map(|_| "pushed to remote".to_string())
        }
        SyncOp::ForcePush => {
            GitRepo::open(&app.repo_path)
                .and_then(|r| r.push(true))
                .map(|_| "force pushed to remote".to_string())
        }
        SyncOp::Fetch => {
            GitRepo::open(&app.repo_path)
                .and_then(|r| r.fetch())
                .map(|_| "fetched remote refs".to_string())
        }
    };

    app.sync_view.status = match result {
        Ok(msg) => SyncStatus::Done(msg),
        Err(e)  => SyncStatus::Error(e.to_string()),
    };
}

fn truncate_msg(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() }
    else { format!("{}…", &s[..max.saturating_sub(1)]) }
}
