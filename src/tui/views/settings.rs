use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::{App, BorderStyle};
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_GREEN, C_YELLOW};

struct SettingItem {
    label: &'static str,
    section: &'static str,
}

const ITEMS: &[SettingItem] = &[
    SettingItem { label: "border style",      section: "appearance" },
    SettingItem { label: "brand color",        section: "appearance" },
    SettingItem { label: "selected bg",        section: "appearance" },
    SettingItem { label: "show history",       section: "views" },
    SettingItem { label: "show remote",        section: "views" },
    SettingItem { label: "show mirror",        section: "views" },
    SettingItem { label: "show workspace",     section: "views" },
    SettingItem { label: "show help",          section: "views" },
];

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(18), Constraint::Min(1)])
        .split(chunks[0]);

    render_sections(f, app, cols[0]);
    render_items(f, app, cols[1]);
    render_status(f, app, chunks[1]);
}

fn render_sections(f: &mut Frame, app: &App, area: Rect) {
    let current_section = ITEMS.get(app.settings_view.idx)
        .map(|i| i.section)
        .unwrap_or("");

    let bc = app.brand_color();
    let focused = !app.sidebar_focused;
    let sections = ["appearance", "views"];
    let items: Vec<ListItem> = sections.iter().map(|s| {
        let is_active = *s == current_section;
        let prefix = if is_active { "█ " } else { "  " };
        let style = if is_active {
            Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(bc)),
            Span::styled(*s, Style::default().fg(if is_active { C_WHITE } else { C_SUBTLE })),
        ])).style(style)
    }).collect();

    let block = Block::default()
        .title(Span::styled(" sections ", Style::default().fg(bc)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(if focused { C_WHITE } else { bc }));
    f.render_widget(List::new(items).block(block), area);
}

fn render_items(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.settings;
    let bc = app.brand_color();
    let focused = !app.sidebar_focused;

    let items: Vec<ListItem> = ITEMS.iter().enumerate().map(|(i, item)| {
        let is_sel = i == app.settings_view.idx;
        let style = if is_sel {
            Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if is_sel { "█ " } else { "  " };
        let val = setting_value(i, s);
        let color = setting_color(i, s);
        let value_span = Span::styled(val, Style::default().fg(color));

        ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(bc)),
            Span::styled(format!("{:<20}", item.label), Style::default().fg(if is_sel { C_WHITE } else { C_SUBTLE })),
            value_span,
        ])).style(style)
    }).collect();

    let mut state = ListState::default();
    state.select(Some(app.settings_view.idx));

    let block = Block::default()
        .title(Span::styled(" settings ",
            if focused { Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD) }
            else { Style::default().fg(bc) }
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(if focused { Style::default().fg(C_WHITE) } else { Style::default().fg(bc) });
    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let bc = app.brand_color();
    let (text, color) = match &app.settings_view.status {
        Some(msg) => (msg.clone(), C_GREEN),
        None => ("[Enter] toggle/edit  [s] save settings".to_string(), C_DIM),
    };

    let block = Block::default()
        .title(Span::styled(" status ", Style::default().fg(bc)))
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

fn setting_value(idx: usize, s: &crate::tui::app::TuiSettings) -> String {
    match idx {
        0  => if s.border_style == BorderStyle::Rounded { "rounded  ╭╮╯╰".to_string() } else { "sharp  ┌┐┘└".to_string() },
        1  => format!("rgb({},{},{})", s.brand_color.0, s.brand_color.1, s.brand_color.2),
        2  => format!("rgb({},{},{})", s.selected_bg.0, s.selected_bg.1, s.selected_bg.2),
        3  => if s.show_history_view   { "visible".to_string() } else { "hidden".to_string() },
        4  => if s.show_remote_view    { "visible".to_string() } else { "hidden".to_string() },
        5  => if s.show_mirror_view    { "visible".to_string() } else { "hidden".to_string() },
        6  => if s.show_workspace_view { "visible".to_string() } else { "hidden".to_string() },
        7  => if s.show_help_view      { "visible".to_string() } else { "hidden".to_string() },
        _  => String::new(),
    }
}

fn setting_color(idx: usize, s: &crate::tui::app::TuiSettings) -> Color {
    match idx {
        0  => C_WHITE,
        1  => Color::Rgb(s.brand_color.0, s.brand_color.1, s.brand_color.2),
        2  => C_WHITE,
        3..=7 => {
            let visible = match idx {
                3 => s.show_history_view,
                4 => s.show_remote_view,
                5 => s.show_mirror_view,
                6 => s.show_workspace_view,
                7 => s.show_help_view,
                _ => true,
            };
            if visible { C_GREEN } else { C_DIM }
        }
        _ => C_SUBTLE,
    }
}
