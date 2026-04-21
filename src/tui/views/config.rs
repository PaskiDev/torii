use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::{App, ConfigScope};
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_GREEN, C_YELLOW, C_CYAN, C_RED};

const SECTIONS: &[&str] = &["user", "auth", "git", "mirror", "snapshot", "ui"];
const SECTION_COLORS: &[ratatui::style::Color] = &[C_CYAN, C_YELLOW, C_GREEN, C_CYAN, C_YELLOW, C_GREEN];

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(16), Constraint::Min(1)])
        .split(chunks[0]);

    render_sections(f, app, cols[0]);
    render_entries(f, app, cols[1]);
    render_status(f, app, chunks[1]);
}

fn render_sections(f: &mut Frame, app: &App, area: Rect) {
    let current_section = app.config_view.entries
        .get(app.config_view.idx)
        .map(|e| e.section.as_str())
        .unwrap_or("");

    let bc = app.brand_color();
    let items: Vec<ListItem> = SECTIONS.iter().enumerate().map(|(i, s)| {
        let is_active = *s == current_section;
        let color = SECTION_COLORS.get(i).copied().unwrap_or(C_SUBTLE);
        let prefix = if is_active { "█ " } else { "  " };
        let style = if is_active {
            Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(bc)),
            Span::styled(*s, Style::default().fg(if is_active { color } else { C_SUBTLE })),
        ])).style(style)
    }).collect();

    let block = Block::default()
        .title(Span::styled(" sections ", Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));
    f.render_widget(List::new(items).block(block), area);
}

fn render_entries(f: &mut Frame, app: &App, area: Rect) {
    let bc = app.brand_color();
    let scope_label = if app.config_view.scope == ConfigScope::Global { "global" } else { "local" };

    let items: Vec<ListItem> = if app.config_view.entries.is_empty() {
        vec![ListItem::new(Span::styled(
            "  no config entries",
            Style::default().fg(C_DIM),
        ))]
    } else {
        app.config_view.entries.iter().enumerate().map(|(i, e)| {
            let is_sel = i == app.config_view.idx;
            let is_editing = is_sel && app.config_view.editing;

            let style = if is_sel {
                Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let prefix = if is_sel { "█ " } else { "  " };
            let key_color = if e.value.contains("[not set]") { C_DIM } else { C_WHITE };

            let value_span = if is_editing {
                let buf = &app.config_view.edit_buf;
                let cur = app.config_view.edit_cursor.min(buf.len());
                let before = &buf[..cur];
                let cursor_char = buf[cur..].chars().next().unwrap_or(' ');
                let after = if buf[cur..].is_empty() { "" } else { &buf[cur + cursor_char.len_utf8()..] };
                Line::from(vec![
                    Span::styled(prefix, Style::default().fg(bc)),
                    Span::styled(format!("{:<32}", &e.key), Style::default().fg(C_CYAN)),
                    Span::styled(before, Style::default().fg(C_WHITE)),
                    Span::styled(cursor_char.to_string(), Style::default().bg(bc).fg(C_WHITE)),
                    Span::styled(after, Style::default().fg(C_WHITE)),
                ])
            } else {
                let value_display = if e.value.contains("[set]") {
                    Span::styled("••••••", Style::default().fg(C_DIM))
                } else if e.value.contains("[not set]") {
                    Span::styled("not set", Style::default().fg(C_RED))
                } else {
                    Span::styled(&e.value, Style::default().fg(key_color))
                };
                Line::from(vec![
                    Span::styled(prefix, Style::default().fg(bc)),
                    Span::styled(format!("{:<32}", &e.key), Style::default().fg(C_SUBTLE)),
                    value_display,
                ])
            };

            ListItem::new(value_span).style(style)
        }).collect()
    };

    let mut state = ListState::default();
    if !app.config_view.entries.is_empty() { state.select(Some(app.config_view.idx)); }

    let title = format!(" config ({}) ", scope_label);
    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));
    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let bc = app.brand_color();
    let (text, color) = if app.config_view.editing {
        ("editing — [Enter] save  [Esc] cancel".to_string(), C_YELLOW)
    } else {
        match &app.config_view.status {
            Some(msg) => (msg.clone(), C_GREEN),
            None => ("ready — [Enter] edit  [Tab] toggle scope".to_string(), C_DIM),
        }
    };

    let block = Block::default()
        .title(Span::styled(" status ", Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::raw(" "),
            Span::styled(text, Style::default().fg(color)),
        ])).block(block),
        area,
    );
}
