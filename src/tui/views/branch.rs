use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::tui::app::App;
use super::super::ui::{C_WHITE, C_SUBTLE, C_GREEN, C_DIM};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let bc = app.brand_color();
    let items: Vec<ListItem> = app.branch_view.branches.iter().enumerate().map(|(i, b)| {
        let is_sel = i == app.branch_view.idx;
        let style = if is_sel {
            Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if is_sel { "█ " } else { "  " };
        let current_marker = if b.is_current { "* " } else { "  " };
        let name_color = if b.is_current { bc } else { C_WHITE };
        let remote_tag = if b.is_remote {
            Span::styled("  remote", Style::default().fg(C_DIM))
        } else {
            Span::raw("")
        };
        let line = Line::from(vec![
            Span::styled(prefix, Style::default().fg(bc)),
            Span::styled(current_marker, Style::default().fg(C_GREEN)),
            Span::styled(&b.name, Style::default().fg(name_color)),
            remote_tag,
        ]);
        ListItem::new(line).style(style)
    }).collect();

    let mut state = ListState::default();
    if !app.branch_view.branches.is_empty() { state.select(Some(app.branch_view.idx)); }

    let block = Block::default()
        .title(Span::styled(
            format!(" branches ({}) ", app.branch_view.branches.len()),
            Style::default().fg(C_SUBTLE),
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));
    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}
