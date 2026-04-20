use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::tui::app::{App, DiffLineKind};
use super::super::ui::{BRAND_COLOR, C_WHITE, C_SUBTLE, C_DIM, C_GREEN, C_RED, C_BORDER};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    let header = Paragraph::new(Line::from(vec![
        Span::styled(" diff  ", Style::default().fg(BRAND_COLOR)),
        Span::styled(&app.diff.title, Style::default().fg(C_WHITE)),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(C_BORDER)));
    f.render_widget(header, chunks[0]);

    let visible_lines: Vec<ListItem> = app.diff.lines
        .iter()
        .skip(app.diff.scroll)
        .map(|line| {
            let (fg, prefix) = match line.kind {
                DiffLineKind::Added   => (C_GREEN,      "+ "),
                DiffLineKind::Removed => (C_RED,        "- "),
                DiffLineKind::Header  => (BRAND_COLOR,  "  "),
                DiffLineKind::Context => (C_SUBTLE,     "  "),
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(fg)),
                Span::styled(&line.content, Style::default().fg(fg)),
            ]))
        })
        .collect();

    f.render_widget(
        List::new(visible_lines)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(C_BORDER))),
        chunks[1],
    );

    let total = app.diff.lines.len();
    let pct = if total == 0 { 0 } else { (app.diff.scroll * 100) / total.max(1) };
    let footer = Line::from(vec![
        Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
        Span::styled(" scroll  ", Style::default().fg(C_SUBTLE)),
        Span::styled("[Esc]", Style::default().fg(BRAND_COLOR)),
        Span::styled(" back  ", Style::default().fg(C_SUBTLE)),
        Span::styled(format!("{}% ({}/{})", pct, app.diff.scroll, total), Style::default().fg(C_DIM)),
    ]);
    f.render_widget(Paragraph::new(footer), chunks[2]);
}
