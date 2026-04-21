use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::App;
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_YELLOW, C_GREEN};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let bc = app.brand_color();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let inner_width = chunks[0].width.saturating_sub(4) as usize;
    let msg_width = inner_width.saturating_sub(20);

    let items: Vec<ListItem> = if app.history_view.reflog.is_empty() {
        vec![ListItem::new(Span::styled(
            "  no reflog entries",
            Style::default().fg(C_DIM),
        ))]
    } else {
        app.history_view.reflog.iter().enumerate().map(|(i, e)| {
            let is_sel = i == app.history_view.idx;
            let style = if is_sel {
                Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if is_sel { "█ " } else { "  " };
            let msg = truncate(&e.message, msg_width);
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(app.brand_color())),
                Span::styled(format!("{} ", &e.id), Style::default().fg(C_YELLOW)),
                Span::styled(format!("{:<width$}", msg, width = msg_width), Style::default().fg(C_WHITE)),
                Span::styled(&e.time, Style::default().fg(C_DIM)),
            ]);
            ListItem::new(line).style(style)
        }).collect()
    };

    let mut state = ListState::default();
    if !app.history_view.reflog.is_empty() { state.select(Some(app.history_view.idx)); }

    let block = Block::default()
        .title(Span::styled(
            format!(" reflog ({} entries) ", app.history_view.reflog.len()),
            Style::default().fg(C_SUBTLE),
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));
    f.render_stateful_widget(List::new(items).block(block), chunks[0], &mut state);

    let status_text = app.history_view.status.as_deref().unwrap_or("ready");
    let status_color = if app.history_view.status.is_some() { C_GREEN } else { C_DIM };
    let status_block = Block::default()
        .title(Span::styled(" status ", Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(status_text, Style::default().fg(status_color)),
        ])).block(status_block),
        chunks[1],
    );
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { return s.to_string(); }
    format!("{}…", &s[..max.saturating_sub(1)])
}
