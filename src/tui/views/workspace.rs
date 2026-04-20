use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::App;
use super::super::ui::{BRAND_COLOR, SELECTED_BG, C_WHITE, C_SUBTLE, C_DIM, C_CYAN, C_YELLOW, C_GREEN, C_RED, C_BORDER};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    if app.workspace_view.workspaces.is_empty() {
        let block = Block::default()
            .title(Span::styled(" workspaces ", Style::default().fg(C_SUBTLE)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_BORDER));
        f.render_widget(
            Paragraph::new(Span::styled(
                "  no workspaces — use `torii workspace add` to create one",
                Style::default().fg(C_DIM),
            )).block(block),
            chunks[0],
        );
    } else {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(24), Constraint::Min(1)])
            .split(chunks[0]);

        // Workspace list (left)
        let ws_items: Vec<ListItem> = app.workspace_view.workspaces.iter().enumerate().map(|(i, ws)| {
            let is_sel = i == app.workspace_view.ws_idx;
            let style = if is_sel {
                Style::default().bg(SELECTED_BG).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if is_sel { "█ " } else { "  " };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(BRAND_COLOR)),
                Span::styled(&ws.name, Style::default().fg(if is_sel { C_WHITE } else { C_SUBTLE })),
                Span::styled(
                    format!(" ({})", ws.repos.len()),
                    Style::default().fg(C_DIM),
                ),
            ]);
            ListItem::new(line).style(style)
        }).collect();

        let mut ws_state = ListState::default();
        ws_state.select(Some(app.workspace_view.ws_idx));

        let ws_block = Block::default()
            .title(Span::styled(" workspaces ", Style::default().fg(C_SUBTLE)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_BORDER));
        f.render_stateful_widget(List::new(ws_items).block(ws_block), cols[0], &mut ws_state);

        // Repo list for selected workspace (right)
        let repo_items: Vec<ListItem> = app.workspace_view.workspaces
            .get(app.workspace_view.ws_idx)
            .map(|ws| {
                ws.repos.iter().map(|r| {
                    let sync_span = if r.ahead > 0 && r.behind > 0 {
                        Span::styled(format!(" ↑{} ↓{}", r.ahead, r.behind), Style::default().fg(C_YELLOW))
                    } else if r.ahead > 0 {
                        Span::styled(format!(" ↑{}", r.ahead), Style::default().fg(C_CYAN))
                    } else if r.behind > 0 {
                        Span::styled(format!(" ↓{}", r.behind), Style::default().fg(C_RED))
                    } else {
                        Span::styled(" synced", Style::default().fg(C_GREEN))
                    };
                    let dirty_span = if r.dirty {
                        Span::styled(" *", Style::default().fg(C_YELLOW))
                    } else {
                        Span::raw("")
                    };
                    let line = Line::from(vec![
                        Span::raw("  "),
                        Span::styled(shorten_path(&r.path, 28), Style::default().fg(C_WHITE)),
                        Span::styled(format!("  {}", r.branch), Style::default().fg(C_GREEN)),
                        sync_span,
                        dirty_span,
                    ]);
                    ListItem::new(line)
                }).collect()
            })
            .unwrap_or_default();

        let repos_block = Block::default()
            .title(Span::styled(" repos ", Style::default().fg(C_SUBTLE)))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(C_BORDER));
        f.render_widget(List::new(repo_items).block(repos_block), cols[1]);
    }

    let status_text = app.workspace_view.status.as_deref().unwrap_or("ready");
    let status_color = if app.workspace_view.status.is_some() { C_GREEN } else { C_DIM };
    let status_block = Block::default()
        .title(Span::styled(" status ", Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(C_BORDER));
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(status_text, Style::default().fg(status_color)),
        ])).block(status_block),
        chunks[1],
    );
}

fn shorten_path(path: &str, max: usize) -> String {
    if path.len() <= max { return path.to_string(); }
    format!("…{}", &path[path.len().saturating_sub(max - 1)..])
}
