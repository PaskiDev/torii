use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::tui::app::App;
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_CYAN, C_YELLOW};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let inner_width = area.width.saturating_sub(4) as usize;
    let msg_width = inner_width.saturating_sub(32);

    let bc = app.brand_color();
    let items: Vec<ListItem> = app.commits.iter().enumerate().map(|(i, c)| {
        let is_sel = i == app.log.idx;
        let style = if is_sel {
            Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if is_sel { "█ " } else { "  " };
        let msg = truncate(&c.message, msg_width);
        let line = Line::from(vec![
            Span::styled(prefix, Style::default().fg(bc)),
            Span::styled(format!("{} ", c.hash), Style::default().fg(C_YELLOW)),
            Span::styled(format!("{:<width$}", msg, width = msg_width), Style::default().fg(C_WHITE)),
            Span::styled(format!(" {:>12}", c.author), Style::default().fg(C_CYAN)),
            Span::styled(format!(" {}", c.time), Style::default().fg(C_DIM)),
        ]);
        ListItem::new(line).style(style)
    }).collect();

    let mut state = ListState::default();
    if !app.commits.is_empty() { state.select(Some(app.log.idx)); }

    let block = Block::default()
        .title(Span::styled(
            format!(" log — {} ({} commits) ", app.branch, app.commits.len()),
            Style::default().fg(C_SUBTLE),
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));
    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { return s.to_string(); }
    format!("{}…", &s[..max.saturating_sub(1)])
}
