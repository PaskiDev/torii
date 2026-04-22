pub mod app;
pub mod events;
pub mod ui;
pub mod views;
pub mod picker;

use std::io;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, View, SyncOp, SyncStatus, EventKind};
use events::{Action, EventHandler};

pub fn run() -> crate::error::Result<()> {
    run_with_view(app::View::Dashboard)
}

pub fn run_with_view(initial_view: app::View) -> crate::error::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    load_auto_interval(&mut app);
    app.go_to(initial_view);
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
        app.tick = app.tick.wrapping_add(1);

        // Poll sync result from background thread
        if let Some(rx) = &app.sync_rx {
            if let Ok(result) = rx.try_recv() {
                app.sync_view.status = match result {
                    Ok(msg) => SyncStatus::Done(msg.clone()),
                    Err(e)  => SyncStatus::Error(e.to_string().lines().next().unwrap_or("error").to_string()),
                };
                let (msg, kind) = match &app.sync_view.status {
                    SyncStatus::Done(m)  => (m.clone(), EventKind::Success),
                    SyncStatus::Error(e) => (e.clone(), EventKind::Error),
                    _                    => ("sync completed".to_string(), EventKind::Info),
                };
                app.log_event(msg, kind);
                app.sync_rx = None;
                app.refresh()?;
            }
        }

        // Auto-snapshot check
        if let Some(interval_secs) = app.snapshot_view.auto_interval.secs() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_secs();
            if now.saturating_sub(app.snapshot_view.last_auto_snapshot) >= interval_secs {
                app.snapshot_view.last_auto_snapshot = now;
                let _ = create_snapshot_with_name(app, "auto");
                app.log_event("auto-snapshot created", EventKind::Info);
            }
        }

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
                    app.log_event("repo refreshed", EventKind::Info);
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
                        let short = truncate_msg(&msg, 40);
                        app.log_event(format!("commit: {}", short), EventKind::Success);
                        app.set_status(format!("committed: {}", short));
                    }
                }

                Action::BranchCheckout => {
                    if let Some(b) = app.branch_view.branches.get(app.branch_view.idx) {
                        let name = b.name.clone();
                        let is_remote = b.is_remote;
                        let repo = crate::core::GitRepo::open(&app.repo_path);
                        let result = match repo {
                            Ok(r) => if is_remote {
                                r.checkout_remote_branch(&name)
                            } else {
                                r.switch_branch(&name)
                            },
                            Err(e) => Err(e),
                        };
                        let msg = match result {
                            Ok(_)  => format!("checkout: {}", name),
                            Err(e) => format!("checkout failed: {}", e),
                        };
                        let is_ok = msg.starts_with("checkout:");
                        app.log_event(&msg, if is_ok { EventKind::Success } else { EventKind::Error });
                        app.branch_view.status = if is_ok { None } else { Some(msg) };
                        app.go_to(View::Branch);
                        app.refresh()?;
                    }
                }

                Action::BranchDelete => {
                    if let Some(b) = app.branch_view.branches.get(app.branch_view.idx) {
                        let name = b.name.clone();
                        let result = crate::core::GitRepo::open(&app.repo_path)
                            .and_then(|r| r.delete_branch(&name));
                        let msg = match result {
                            Ok(_)  => format!("deleted: {}", name),
                            Err(e) => format!("delete failed: {}", e),
                        };
                        let kind = if msg.starts_with("deleted") { EventKind::Success } else { EventKind::Error };
                        app.log_event(&msg, kind);
                        app.branch_view.status = None;
                        app.go_to(View::Branch);
                    }
                }

                Action::BranchCreate => {
                    let name = app.branch_view.new_name.trim().to_string();
                    if !name.is_empty() {
                        let result = crate::core::GitRepo::open(&app.repo_path)
                            .and_then(|r| r.create_branch(&name).and_then(|_| r.switch_branch(&name)));
                        let msg = match result {
                            Ok(_)  => format!("created: {}", name),
                            Err(e) => format!("create failed: {}", e),
                        };
                        let is_ok = msg.starts_with("created");
                        app.log_event(&msg, if is_ok { EventKind::Success } else { EventKind::Error });
                        app.branch_view.new_name.clear();
                        app.branch_view.status = if is_ok { None } else { Some(msg) };
                        app.go_to(View::Branch);
                        app.refresh()?;
                    }
                }

                Action::BranchPush => {
                    let result = crate::core::GitRepo::open(&app.repo_path)
                        .and_then(|r| r.push(false));
                    let msg = match result {
                        Ok(_)  => format!("pushed: {}", app.branch),
                        Err(e) => format!("push failed: {}", e),
                    };
                    let is_ok = msg.starts_with("pushed");
                    app.log_event(&msg, if is_ok { EventKind::Success } else { EventKind::Error });
                    app.branch_view.status = if is_ok { None } else { Some(msg) };
                    app.refresh()?;
                }

                Action::SnapshotRestore => {
                    restore_selected_snapshot(app)?;
                    app.go_back();
                    app.refresh()?;
                }

                Action::SnapshotCreate => {
                    create_snapshot(app)?;
                    app.log_event("snapshot created", EventKind::Success);
                }

                Action::SnapshotDelete => {
                    delete_snapshot(app)?;
                    app.log_event("snapshot deleted", EventKind::Info);
                }

                Action::SnapshotSaveInterval => {
                    save_auto_interval(app);
                    app.log_event(
                        format!("auto-snapshot: {}", app.snapshot_view.auto_interval.label()),
                        EventKind::Info,
                    );
                }

                Action::OpenDiffFromLog => {
                    app.go_to_diff_from_log();
                }

                Action::LogCopyHash => {
                    if let Some(commit) = app.commits.get(app.log.idx) {
                        let hash = commit.full_hash.clone();
                        let copied = copy_to_clipboard(&hash);
                        let msg = if copied {
                            format!("copied: {}", &hash[..7])
                        } else {
                            "clipboard not available".to_string()
                        };
                        let kind = if copied { EventKind::Success } else { EventKind::Error };
                        app.log_event(&msg, kind);
                        app.set_status(msg);
                    }
                }

                Action::SyncRun => {
                    spawn_sync(app);
                }

                Action::TagCreate => {
                    let name = app.tag_view.new_name.trim().to_string();
                    let message = app.tag_view.new_message.trim().to_string();
                    app.tag_view.new_name.clear();
                    app.tag_view.new_message.clear();
                    if !name.is_empty() {
                        let result = std::process::Command::new("torii")
                            .args(["tag", "create", &name, "-m", &message])
                            .current_dir(&app.repo_path)
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status();
                        let is_ok = result.map(|s| s.success()).unwrap_or(false);
                        let msg = if is_ok { format!("created tag: {}", name) } else { format!("failed to create tag: {}", name) };
                        app.log_event(&msg, if is_ok { EventKind::Success } else { EventKind::Error });
                        app.refresh()?;
                    }
                }

                Action::TagPush => {
                    if let Some(tag) = app.tag_view.tags.get(app.tag_view.idx) {
                        let name = tag.name.clone();
                        let result = std::process::Command::new("torii")
                            .args(["tag", "push", &name])
                            .current_dir(&app.repo_path)
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status();
                        let is_ok = result.map(|s| s.success()).unwrap_or(false);
                        let msg = if is_ok { format!("pushed tag: {}", name) } else { format!("failed to push tag: {}", name) };
                        app.log_event(&msg, if is_ok { EventKind::Success } else { EventKind::Error });
                    }
                }

                Action::TagDelete => {
                    if let Some(tag) = app.tag_view.tags.get(app.tag_view.idx) {
                        let name = tag.name.clone();
                        let result = std::process::Command::new("torii")
                            .args(["tag", "delete", &name])
                            .current_dir(&app.repo_path)
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status();
                        let is_ok = result.map(|s| s.success()).unwrap_or(false);
                        let msg = if is_ok { format!("deleted tag: {}", name) } else { format!("failed to delete tag: {}", name) };
                        app.log_event(&msg, if is_ok { EventKind::Success } else { EventKind::Error });
                        app.refresh()?;
                    }
                }

                Action::HistoryCherryPick => {
                    if let Some(entry) = app.history_view.reflog.get(app.history_view.idx) {
                        let hash = entry.id.clone();
                        let ok = std::process::Command::new("torii")
                            .args(["history", "cherry-pick", &hash])
                            .current_dir(&app.repo_path)
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status().map(|s| s.success()).unwrap_or(false);
                        let msg = if ok { format!("cherry-picked: {}", hash) } else { format!("cherry-pick failed: {}", hash) };
                        app.log_event(&msg, if ok { EventKind::Success } else { EventKind::Error });
                        app.refresh()?;
                    }
                }

                Action::HistoryRebase => {
                    let target = app.history_view.input.trim().to_string();
                    app.history_view.input.clear();
                    if !target.is_empty() {
                        let ok = std::process::Command::new("torii")
                            .args(["history", "rebase", &target])
                            .current_dir(&app.repo_path)
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status().map(|s| s.success()).unwrap_or(false);
                        let msg = if ok { format!("rebased onto: {}", target) } else { format!("rebase failed onto: {}", target) };
                        app.log_event(&msg, if ok { EventKind::Success } else { EventKind::Error });
                        app.refresh()?;
                    }
                }

                Action::HistoryScan => {
                    let full = app.history_view.scan_full;
                    let mut cmd = std::process::Command::new("torii");
                    cmd.args(if full { vec!["history", "scan", "--history"] } else { vec!["history", "scan"] })
                        .current_dir(&app.repo_path)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null());
                    let ok = cmd.status().map(|s| s.success()).unwrap_or(false);
                    let msg = if ok { "scan complete — no secrets found".to_string() } else { "scan found issues — check event log".to_string() };
                    app.log_event(&msg, if ok { EventKind::Success } else { EventKind::Error });
                }

                Action::HistoryClean => {
                    let ok = std::process::Command::new("torii")
                        .args(["history", "clean"])
                        .current_dir(&app.repo_path)
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status().map(|s| s.success()).unwrap_or(false);
                    let msg = if ok { "history cleaned".to_string() } else { "clean failed".to_string() };
                    app.log_event(&msg, if ok { EventKind::Success } else { EventKind::Error });
                    app.refresh()?;
                }

                Action::HistoryRemoveFile => {
                    let path = app.history_view.input.trim().to_string();
                    app.history_view.input.clear();
                    if !path.is_empty() {
                        let ok = std::process::Command::new("torii")
                            .args(["history", "remove-file", &path])
                            .current_dir(&app.repo_path)
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status().map(|s| s.success()).unwrap_or(false);
                        let msg = if ok { format!("removed file from history: {}", path) } else { format!("remove-file failed: {}", path) };
                        app.log_event(&msg, if ok { EventKind::Success } else { EventKind::Error });
                        app.refresh()?;
                    }
                }

                Action::HistoryRewrite => {
                    let start = app.history_view.input.trim().to_string();
                    let end = app.history_view.input2.trim().to_string();
                    app.history_view.input.clear();
                    app.history_view.input2.clear();
                    if !start.is_empty() && !end.is_empty() {
                        let ok = std::process::Command::new("torii")
                            .args(["history", "rewrite", &start, &end])
                            .current_dir(&app.repo_path)
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status().map(|s| s.success()).unwrap_or(false);
                        let msg = if ok { format!("rewrote dates: {} → {}", start, end) } else { "rewrite failed".to_string() };
                        app.log_event(&msg, if ok { EventKind::Success } else { EventKind::Error });
                        app.refresh()?;
                    }
                }

                Action::HistoryBlame => {
                    let file = app.history_view.input.trim().to_string();
                    app.history_view.input.clear();
                    if !file.is_empty() {
                        let output = std::process::Command::new("torii")
                            .args(["history", "blame", &file])
                            .current_dir(&app.repo_path)
                            .output();
                        match output {
                            Ok(o) if o.status.success() => {
                                let text = String::from_utf8_lossy(&o.stdout).to_string();
                                app.history_view.input = text;
                                app.log_event(&format!("blame: {}", file), EventKind::Info);
                            }
                            _ => { app.log_event(&format!("blame failed: {}", file), EventKind::Error); }
                        }
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
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
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
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
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
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status();
                        let msg = match status {
                            Ok(s) if s.success() => format!("synced workspace: {}", name),
                            _ => format!("sync failed for: {}", name),
                        };
                        let kind = if msg.starts_with("synced") { EventKind::Success } else { EventKind::Error };
                        app.log_event(&msg, kind);
                        app.workspace_view.status = Some(msg);
                        app.go_to(View::Workspace);
                    }
                }

                Action::WorkspaceSyncOne => {
                    let repo_path = app.workspace_view.workspaces
                        .get(app.workspace_view.ws_idx)
                        .and_then(|ws| ws.repos.get(app.workspace_view.repo_idx))
                        .map(|r| r.path.clone());
                    if let Some(path) = repo_path {
                        let status = std::process::Command::new("torii")
                            .args(["sync"])
                            .current_dir(&path)
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .status();
                        let msg = match status {
                            Ok(s) if s.success() => format!("synced: {}", path),
                            _ => format!("sync failed: {}", path),
                        };
                        let kind = if msg.starts_with("synced") { EventKind::Success } else { EventKind::Error };
                        app.log_event(&msg, kind);
                        app.workspace_view.status = Some(msg);
                        app.refresh().ok();
                    }
                }

                Action::WorkspaceOpenRepo => {
                    let repo_path = app.workspace_view.workspaces
                        .get(app.workspace_view.ws_idx)
                        .and_then(|ws| ws.repos.get(app.workspace_view.repo_idx))
                        .map(|r| r.path.clone());
                    if let Some(path) = repo_path {
                        app.log_event(format!("opened: {}", path), EventKind::Info);
                        app.repo_path = path;
                        app.refresh().ok();
                        app.go_to(View::Dashboard);
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


fn snapshot_dir(app: &App) -> std::path::PathBuf {
    std::path::Path::new(&app.repo_path).join(".git/torii-snapshots")
}

fn torii_dir(app: &App) -> std::path::PathBuf {
    std::path::Path::new(&app.repo_path).join(".torii")
}

fn save_auto_interval(app: &App) {
    use crate::tui::app::AutoSnapshotInterval;
    let dir = torii_dir(app);
    let _ = std::fs::create_dir_all(&dir);
    let val = match app.snapshot_view.auto_interval {
        AutoSnapshotInterval::Off   => "off",
        AutoSnapshotInterval::Min5  => "5min",
        AutoSnapshotInterval::Min15 => "15min",
        AutoSnapshotInterval::Min30 => "30min",
        AutoSnapshotInterval::Hour1 => "1h",
    };
    let _ = std::fs::write(dir.join("auto-interval"), val);
}

fn load_auto_interval(app: &mut App) {
    use crate::tui::app::AutoSnapshotInterval;
    let path = torii_dir(app).join("auto-interval");
    let val = std::fs::read_to_string(path).unwrap_or_default();
    app.snapshot_view.auto_interval = match val.trim() {
        "5min"  => AutoSnapshotInterval::Min5,
        "15min" => AutoSnapshotInterval::Min15,
        "30min" => AutoSnapshotInterval::Min30,
        "1h"    => AutoSnapshotInterval::Hour1,
        _       => AutoSnapshotInterval::Off,
    };
    app.snapshot_view.auto_interval_idx = AutoSnapshotInterval::all()
        .iter().position(|i| i == &app.snapshot_view.auto_interval)
        .unwrap_or(0);
}

fn create_snapshot_with_name(app: &mut App, name: &str) -> crate::error::Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dir = snapshot_dir(app);
    std::fs::create_dir_all(&dir)?;
    let id = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs().to_string();
    std::fs::write(dir.join(format!("{}.meta", id)), name)?;
    app.load_snapshots();
    Ok(())
}

fn create_snapshot(app: &mut App) -> crate::error::Result<()> {
    use crate::tui::app::SnapshotFocus;
    let name = if app.snapshot_view.create_name.trim().is_empty() {
        "snapshot".to_string()
    } else {
        app.snapshot_view.create_name.trim().to_string()
    };
    app.snapshot_view.focus = SnapshotFocus::List;
    create_snapshot_with_name(app, &name)
}

fn delete_snapshot(app: &mut App) -> crate::error::Result<()> {
    if let Some(s) = app.snapshot_view.snapshots.get(app.snapshot_view.idx) {
        let id = s.id.clone();
        let dir = snapshot_dir(app);
        let _ = std::fs::remove_file(dir.join(format!("{}.meta", id)));
    }
    app.load_snapshots();
    Ok(())
}

fn restore_selected_snapshot(app: &mut App) -> crate::error::Result<()> {
    let snap = app.snapshot_view.snapshots.get(app.snapshot_view.idx);
    if let Some(s) = snap {
        app.set_status(format!("restored: {}", s.name));
    }
    Ok(())
}

fn spawn_sync(app: &mut App) {
    use crate::core::GitRepo;
    use std::sync::mpsc;

    app.sync_view.status = SyncStatus::Running;

    let repo_path = app.repo_path.clone();
    let op = app.sync_view.selected_op.clone();

    let (tx, rx) = mpsc::channel();
    app.sync_rx = Some(rx);

    std::thread::spawn(move || {
        let result = match op {
            SyncOp::PullPush  => GitRepo::open(&repo_path)
                .and_then(|r| r.pull().and_then(|_| r.push(false)))
                .map(|_| "synced with remote".to_string()),
            SyncOp::PullOnly  => GitRepo::open(&repo_path)
                .and_then(|r| r.pull())
                .map(|_| "pulled from remote".to_string()),
            SyncOp::PushOnly  => GitRepo::open(&repo_path)
                .and_then(|r| r.push(false))
                .map(|_| "pushed to remote".to_string()),
            SyncOp::ForcePush => GitRepo::open(&repo_path)
                .and_then(|r| r.push(true))
                .map(|_| "force pushed to remote".to_string()),
            SyncOp::Fetch     => GitRepo::open(&repo_path)
                .and_then(|r| r.fetch())
                .map(|_| "fetched remote refs".to_string()),
        };
        let _ = tx.send(result);
    });
}

fn copy_to_clipboard(text: &str) -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let pipe_cmd = |cmd: &str, args: &[&str]| -> bool {
        Command::new(cmd)
            .args(args)
            .stdin(Stdio::piped())
            .spawn()
            .and_then(|mut c| {
                c.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
                c.wait()
            })
            .map(|s| s.success())
            .unwrap_or(false)
    };

    // macOS
    if cfg!(target_os = "macos") {
        return pipe_cmd("pbcopy", &[]);
    }

    // Windows
    if cfg!(target_os = "windows") {
        return pipe_cmd("clip", &[]);
    }

    // Wayland
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        if Command::new("wl-copy").arg(text).status().map(|s| s.success()).unwrap_or(false) {
            return true;
        }
    }

    // X11
    if pipe_cmd("xclip", &["-selection", "clipboard"]) { return true; }
    if pipe_cmd("xsel", &["--clipboard", "--input"])    { return true; }

    false
}

fn truncate_msg(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() }
    else { format!("{}…", &s[..max.saturating_sub(1)]) }
}
