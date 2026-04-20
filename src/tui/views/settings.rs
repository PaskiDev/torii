use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::{App, BorderStyle};
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_GREEN, C_YELLOW, C_BORDER};

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
    SettingItem { label: "key: files",         section: "keybinds" },
    SettingItem { label: "key: save",          section: "keybinds" },
    SettingItem { label: "key: sync",          section: "keybinds" },
    SettingItem { label: "key: snapshot",      section: "keybinds" },
    SettingItem { label: "key: log",           section: "keybinds" },
    SettingItem { label: "key: branch",        section: "keybinds" },
    SettingItem { label: "key: tags",          section: "keybinds" },
    SettingItem { label: "key: history",       section: "keybinds" },
    SettingItem { label: "key: remote",        section: "keybinds" },
    SettingItem { label: "key: mirror",        section: "keybinds" },
    SettingItem { label: "key: workspace",     section: "keybinds" },
    SettingItem { label: "key: config",        section: "keybinds" },
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
    let sections = ["appearance", "views", "keybinds"];
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
        .title(Span::styled(" sections ", Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(C_BORDER));
    f.render_widget(List::new(items).block(block), area);
}

fn render_items(f: &mut Frame, app: &App, area: Rect) {
    let s = &app.settings;
    let editing_idx = app.settings_view.editing_keybind;
    let bc = app.brand_color();

    let items: Vec<ListItem> = ITEMS.iter().enumerate().map(|(i, item)| {
        let is_sel = i == app.settings_view.idx;
        let is_editing = editing_idx == Some(i);
        let style = if is_sel {
            Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if is_sel { "█ " } else { "  " };

        let value_span = if is_editing {
            Span::styled("press new key…", Style::default().fg(C_YELLOW).add_modifier(Modifier::BOLD))
        } else {
            let val = setting_value(i, s);
            let color = setting_color(i, s);
            Span::styled(val, Style::default().fg(color))
        };

        ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(bc)),
            Span::styled(format!("{:<20}", item.label), Style::default().fg(if is_sel { C_WHITE } else { C_SUBTLE })),
            value_span,
        ])).style(style)
    }).collect();

    let mut state = ListState::default();
    state.select(Some(app.settings_view.idx));

    let block = Block::default()
        .title(Span::styled(" settings ", Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(C_BORDER));
    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let (text, color) = if app.settings_view.editing_keybind.is_some() {
        ("press any key to assign  [Esc] cancel".to_string(), C_YELLOW)
    } else {
        match &app.settings_view.status {
            Some(msg) => (msg.clone(), C_GREEN),
            None => ("[Enter] toggle/edit  [s] save settings".to_string(), C_DIM),
        }
    };

    let block = Block::default()
        .title(Span::styled(" status ", Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(C_BORDER));
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
        8  => s.keybind_files.to_string(),
        9  => s.keybind_save.to_string(),
        10 => s.keybind_sync.to_string(),
        11 => s.keybind_snapshot.to_string(),
        12 => s.keybind_log.to_string(),
        13 => s.keybind_branch.to_string(),
        14 => s.keybind_tag.to_string(),
        15 => s.keybind_history.to_string(),
        16 => s.keybind_remote.to_string(),
        17 => s.keybind_mirror.to_string(),
        18 => s.keybind_workspace.to_string(),
        19 => s.keybind_config.to_string(),
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
        _ => C_YELLOW,
    }
}
