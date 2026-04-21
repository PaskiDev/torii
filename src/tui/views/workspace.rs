use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::{App, WorkspaceFocus};
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_CYAN, C_YELLOW, C_GREEN, C_RED};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    if app.workspace_view.workspaces.is_empty() {
        let block = Block::default()
            .title(Span::styled(" workspaces ", Style::default().fg(C_SUBTLE)))
            .borders(Borders::ALL).border_type(app.border_type())
            .border_style(Style::default().fg(app.brand_color()));
        f.render_widget(
            Paragraph::new(Span::styled(
                "  no workspaces — run `torii tui` outside a repo to create one",
                Style::default().fg(C_DIM),
            )).block(block),
            chunks[0],
        );
    } else {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(26), Constraint::Min(1)])
            .split(chunks[0]);

        let bc = app.brand_color();
        let focus_ws = app.workspace_view.focus == WorkspaceFocus::Workspaces;
        let focus_repos = !focus_ws;

        // ── Lista de workspaces (izquierda) ──────────────────────────────────
        let ws_items: Vec<ListItem> = app.workspace_view.workspaces.iter().enumerate().map(|(i, ws)| {
            let is_sel = i == app.workspace_view.ws_idx;
            let is_active = is_sel && focus_ws;
            let style = if is_active {
                Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
            } else if is_sel {
                Style::default().bg(app.selected_bg())
            } else {
                Style::default()
            };
            let prefix = if is_active { "█ " } else if is_sel { "▶ " } else { "  " };

            // Cuenta repos sucios
            let dirty = ws.repos.iter().filter(|r| r.dirty).count();
            let ahead_total: usize = ws.repos.iter().map(|r| r.ahead).sum();
            let behind_total: usize = ws.repos.iter().map(|r| r.behind).sum();
            let sync_color = if ahead_total > 0 || behind_total > 0 { C_YELLOW } else { C_GREEN };
            let sync_sym = if ahead_total > 0 && behind_total > 0 { "⇅" }
                else if ahead_total > 0 { "↑" }
                else if behind_total > 0 { "↓" }
                else { "✓" };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(bc)),
                Span::styled(format!("{:<18}", &ws.name), Style::default().fg(if is_sel { C_WHITE } else { C_SUBTLE })),
                Span::styled(format!("{}", ws.repos.len()), Style::default().fg(C_DIM)),
                Span::styled(format!(" {} ", sync_sym), Style::default().fg(sync_color)),
                if dirty > 0 { Span::styled(format!("*{}", dirty), Style::default().fg(C_YELLOW)) }
                else { Span::raw("") },
            ]);
            ListItem::new(line).style(style)
        }).collect();

        let mut ws_state = ListState::default();
        ws_state.select(Some(app.workspace_view.ws_idx));

        let ws_border_color = if focus_ws { C_WHITE } else { bc };
        let ws_block = Block::default()
            .title(Span::styled(" workspaces ", Style::default().fg(if focus_ws { C_WHITE } else { bc })))
            .borders(Borders::ALL).border_type(app.border_type())
            .border_style(Style::default().fg(ws_border_color));
        f.render_stateful_widget(List::new(ws_items).block(ws_block), cols[0], &mut ws_state);

        // ── Lista de repos (derecha) ─────────────────────────────────────────
        let repos_border_color = if focus_repos { C_WHITE } else { bc };

        let repo_items: Vec<ListItem> = app.workspace_view.workspaces
            .get(app.workspace_view.ws_idx)
            .map(|ws| {
                ws.repos.iter().enumerate().map(|(i, r)| {
                    let is_sel = focus_repos && i == app.workspace_view.repo_idx;
                    let style = if is_sel {
                        Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    let prefix = if is_sel { "█ " } else { "  " };

                    let name = std::path::Path::new(&r.path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| r.path.clone());

                    let sync_span = if r.ahead > 0 && r.behind > 0 {
                        Span::styled(format!(" ↑{} ↓{}", r.ahead, r.behind), Style::default().fg(C_YELLOW))
                    } else if r.ahead > 0 {
                        Span::styled(format!(" ↑{} ahead", r.ahead), Style::default().fg(C_CYAN))
                    } else if r.behind > 0 {
                        Span::styled(format!(" ↓{} behind", r.behind), Style::default().fg(C_RED))
                    } else {
                        Span::styled(" synced", Style::default().fg(C_GREEN))
                    };

                    let dirty_span = if r.dirty {
                        Span::styled(" *", Style::default().fg(C_YELLOW).add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw("")
                    };

                    let line = Line::from(vec![
                        Span::styled(prefix, Style::default().fg(bc)),
                        Span::styled(format!("{:<22}", name), Style::default().fg(if is_sel { C_WHITE } else { C_SUBTLE })),
                        Span::styled(format!(" {:<14}", &r.branch), Style::default().fg(C_GREEN)),
                        sync_span,
                        dirty_span,
                    ]);
                    ListItem::new(line).style(style)
                }).collect()
            })
            .unwrap_or_default();

        let mut repo_state = ListState::default();
        if focus_repos {
            repo_state.select(Some(app.workspace_view.repo_idx));
        }

        let ws_name = app.workspace_view.workspaces
            .get(app.workspace_view.ws_idx)
            .map(|ws| ws.name.as_str())
            .unwrap_or("");
        let repos_block = Block::default()
            .title(Span::styled(
                format!(" {} — repos ", ws_name),
                Style::default().fg(if focus_repos { C_WHITE } else { bc }),
            ))
            .borders(Borders::ALL).border_type(app.border_type())
            .border_style(Style::default().fg(repos_border_color));
        f.render_stateful_widget(List::new(repo_items).block(repos_block), cols[1], &mut repo_state);
    }

    // ── Status ───────────────────────────────────────────────────────────────
    let (status_text, status_color) = match &app.workspace_view.status {
        Some(msg) => (msg.as_str(), C_GREEN),
        None => match app.workspace_view.focus {
            WorkspaceFocus::Workspaces => ("  [→/l] repos  [Enter] sync workspace", C_DIM),
            WorkspaceFocus::Repos      => ("  [Enter] open repo  [s] sync repo  [S] sync workspace  [←/h] workspaces", C_DIM),
        },
    };

    let status_block = Block::default()
        .title(Span::styled(" status ", Style::default().fg(app.brand_color())))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(app.brand_color()));
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(status_text, Style::default().fg(status_color)),
        ])).block(status_block),
        chunks[1],
    );
}
