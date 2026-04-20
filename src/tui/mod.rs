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

use app::{App, View, SyncOp, SyncStatus, ConfigScope, BorderStyle};
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

                Action::SidebarUp => { app.sidebar_up(); }
                Action::SidebarDown => { app.sidebar_down(); }
                Action::SidebarEnter => {
                    app.sidebar_focused = false;
                    app.sidebar_enter();
                }

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

                Action::TagPush => {
                    if let Some(tag) = app.tag_view.tags.get(app.tag_view.idx) {
                        let name = tag.name.clone();
                        let status = std::process::Command::new("git")
                            .args(["push", "origin", &name])
                            .current_dir(&app.repo_path)
                            .status();
                        app.tag_view.status = Some(match status {
                            Ok(s) if s.success() => format!("pushed tag: {}", name),
                            _ => format!("failed to push tag: {}", name),
                        });
                    }
                }

                Action::TagDelete => {
                    if let Some(tag) = app.tag_view.tags.get(app.tag_view.idx) {
                        let name = tag.name.clone();
                        let status = std::process::Command::new("git")
                            .args(["tag", "-d", &name])
                            .current_dir(&app.repo_path)
                            .status();
                        app.tag_view.status = Some(match status {
                            Ok(s) if s.success() => format!("deleted tag: {}", name),
                            _ => format!("failed to delete tag: {}", name),
                        });
                        app.go_to(View::Tag);
                    }
                }

                Action::HistoryCherryPick => {
                    if let Some(entry) = app.history_view.reflog.get(app.history_view.idx) {
                        let hash = entry.id.clone();
                        let status = std::process::Command::new("git")
                            .args(["cherry-pick", &hash])
                            .current_dir(&app.repo_path)
                            .status();
                        app.history_view.status = Some(match status {
                            Ok(s) if s.success() => format!("cherry-picked: {}", hash),
                            _ => format!("cherry-pick failed: {}", hash),
                        });
                        app.refresh()?;
                    }
                }

                Action::RemoteInfo => {
                    if let Some(remote) = app.remote_view.remotes.get(app.remote_view.idx) {
                        app.remote_view.status = Some(format!("{} → {}", remote.name, remote.url));
                    }
                }

                Action::MirrorSync => {
                    let status = std::process::Command::new("torii")
                        .args(["mirror", "sync"])
                        .current_dir(&app.repo_path)
                        .status();
                    app.mirror_view.status = Some(match status {
                        Ok(s) if s.success() => "synced all mirrors".to_string(),
                        _ => "mirror sync failed".to_string(),
                    });
                }

                Action::ConfigEdit => {} // editing already started in handle_config

                Action::ConfigSave => {
                    let idx = app.config_view.idx;
                    if let Some(entry) = app.config_view.entries.get(idx) {
                        let key = entry.key.clone();
                        let val = app.config_view.edit_buf.clone();
                        let scope_flag = if app.config_view.scope == app::ConfigScope::Local {
                            "--local"
                        } else {
                            "--global"
                        };
                        let status = std::process::Command::new("torii")
                            .args(["config", "set", &key, &val, scope_flag])
                            .status();
                        app.config_view.editing = false;
                        app.config_view.status = Some(match status {
                            Ok(s) if s.success() => format!("saved: {} = {}", key, val),
                            _ => format!("failed to save: {}", key),
                        });
                        app.go_to(View::Config);
                    }
                }

                Action::ConfigToggleScope => {
                    app.config_view.scope = if app.config_view.scope == app::ConfigScope::Global {
                        app::ConfigScope::Local
                    } else {
                        app::ConfigScope::Global
                    };
                    app.go_to(View::Config);
                }

                Action::SettingsToggle => {
                    let idx = app.settings_view.idx;
                    match idx {
                        0 => {
                            app.settings.border_style = if app.settings.border_style == app::BorderStyle::Rounded {
                                app::BorderStyle::Sharp
                            } else {
                                app::BorderStyle::Rounded
                            };
                        }
                        3 => app.settings.show_history_view   = !app.settings.show_history_view,
                        4 => app.settings.show_remote_view    = !app.settings.show_remote_view,
                        5 => app.settings.show_mirror_view    = !app.settings.show_mirror_view,
                        6 => app.settings.show_workspace_view = !app.settings.show_workspace_view,
                        7 => app.settings.show_help_view      = !app.settings.show_help_view,
                        8..=19 => { app.settings_view.editing_keybind = Some(idx); }
                        _ => {}
                    }
                }

                Action::SettingsSave => {
                    app.settings.save();
                    app.settings_view.status = Some("settings saved".to_string());
                }

                Action::SettingsEditKeybind => {}

                Action::WorkspaceSync => {
                    if let Some(ws) = app.workspace_view.workspaces.get(app.workspace_view.ws_idx) {
                        let name = ws.name.clone();
                        let status = std::process::Command::new("torii")
                            .args(["workspace", "sync", &name])
                            .status();
                        app.workspace_view.status = Some(match status {
                            Ok(s) if s.success() => format!("synced workspace: {}", name),
                            _ => format!("sync failed for: {}", name),
                        });
                        app.go_to(View::Workspace);
                    }
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
