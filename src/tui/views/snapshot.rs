use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::tui::app::App;
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_YELLOW, C_BORDER};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = if app.snapshot_view.snapshots.is_empty() {
        vec![ListItem::new(Span::styled(
            "  no snapshots — run `torii snapshot save` to create one",
            Style::default().fg(C_DIM),
        ))]
    } else {
        app.snapshot_view.snapshots.iter().enumerate().map(|(i, s)| {
            let is_sel = i == app.snapshot_view.idx;
            let style = if is_sel {
                Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if is_sel { "█ " } else { "  " };
            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(app.brand_color())),
                Span::styled(&s.name, Style::default().fg(C_WHITE)),
                Span::raw("  "),
                Span::styled(&s.id, Style::default().fg(C_YELLOW)),
                Span::raw("  "),
                Span::styled(&s.time, Style::default().fg(C_DIM)),
            ]);
            ListItem::new(line).style(style)
        }).collect()
    };

    let mut state = ListState::default();
    if !app.snapshot_view.snapshots.is_empty() { state.select(Some(app.snapshot_view.idx)); }

    let block = Block::default()
        .title(Span::styled(
            format!(" snapshots ({}) ", app.snapshot_view.snapshots.len()),
            Style::default().fg(C_SUBTLE),
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(C_BORDER));
    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}
