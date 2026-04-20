use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::App;
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_CYAN, C_GREEN, C_BORDER};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let items: Vec<ListItem> = if app.remote_view.remotes.is_empty() {
        vec![ListItem::new(Span::styled(
            "  no remotes configured",
            Style::default().fg(C_DIM),
        ))]
    } else {
        app.remote_view.remotes.iter().enumerate().map(|(i, r)| {
            let is_sel = i == app.remote_view.idx;
            let style = if is_sel {
                Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if is_sel { "█ " } else { "  " };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(app.brand_color())),
                Span::styled(format!("{:<12}", &r.name), Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled(&r.url, Style::default().fg(C_CYAN)),
            ]);
            ListItem::new(line).style(style)
        }).collect()
    };

    let mut state = ListState::default();
    if !app.remote_view.remotes.is_empty() { state.select(Some(app.remote_view.idx)); }

    let block = Block::default()
        .title(Span::styled(
            format!(" remotes ({}) ", app.remote_view.remotes.len()),
            Style::default().fg(C_SUBTLE),
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(C_BORDER));
    f.render_stateful_widget(List::new(items).block(block), chunks[0], &mut state);

    let status_text = app.remote_view.status.as_deref().unwrap_or("ready");
    let status_color = if app.remote_view.status.is_some() { C_GREEN } else { C_DIM };
    let status_block = Block::default()
        .title(Span::styled(" status ", Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(C_BORDER));
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(status_text, Style::default().fg(status_color)),
        ])).block(status_block),
        chunks[1],
    );
}
