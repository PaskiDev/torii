use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

use crate::tui::app::App;
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_CYAN, C_YELLOW, C_GREEN, C_RED};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let bc = app.brand_color();
    let focused = !app.sidebar_focused;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);

    // ── Commit list ───────────────────────────────────────────────────────────
    let inner_width = chunks[0].width.saturating_sub(4) as usize;
    let msg_width = inner_width.saturating_sub(32);

    let display_indices: Vec<usize> = if app.log.filtered.is_empty() && app.log.search_query.is_empty() {
        (0..app.commits.len()).collect()
    } else {
        app.log.filtered.clone()
    };

    let items: Vec<ListItem> = display_indices.iter().map(|&i| {
        let c = &app.commits[i];
        let is_sel = i == app.log.idx;
        let style = if is_sel {
            Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if is_sel { "█ " } else { "  " };
        let msg = truncate(&c.message, msg_width);
        let msg_color = if !app.log.search_query.is_empty() && !app.log.filtered.is_empty() {
            C_GREEN
        } else {
            C_WHITE
        };
        let line = Line::from(vec![
            Span::styled(prefix, Style::default().fg(bc)),
            Span::styled(format!("{} ", c.hash), Style::default().fg(C_YELLOW)),
            Span::styled(format!("{:<width$}", msg, width = msg_width), Style::default().fg(if is_sel { C_WHITE } else { msg_color })),
            Span::styled(format!(" {:>10}", truncate(&c.author, 10)), Style::default().fg(C_CYAN)),
            Span::styled(format!(" {}", c.time), Style::default().fg(C_DIM)),
        ]);
        ListItem::new(line).style(style)
    }).collect();

    let sel_pos = display_indices.iter().position(|&i| i == app.log.idx);
    let mut state = ListState::default();
    if let Some(pos) = sel_pos { state.select(Some(pos)); }

    let total = app.commits.len();
    let loaded_hint = if app.log.all_loaded { String::new() } else { "  ↓ more".to_string() };
    let title = if app.log.search_mode {
        format!(" log — search: {}█ ", app.log.search_query)
    } else if !app.log.search_query.is_empty() {
        format!(" log — \"{}\"  {} matches ", app.log.search_query, display_indices.len())
    } else {
        format!(" log — {} ({} commits){} ", app.branch, total, loaded_hint)
    };

    let title_color = if app.log.search_mode { C_YELLOW } else if focused { C_WHITE } else { C_SUBTLE };

    let list_block = Block::default()
        .title(Span::styled(title, Style::default().fg(title_color).add_modifier(
            if focused || app.log.search_mode { Modifier::BOLD } else { Modifier::empty() }
        )))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(if focused { Style::default().fg(C_WHITE) } else { Style::default().fg(bc) });
    f.render_stateful_widget(List::new(items).block(list_block), chunks[0], &mut state);

    // ── Files panel ───────────────────────────────────────────────────────────
    let file_items: Vec<ListItem> = if app.log.commit_files.is_empty() {
        vec![ListItem::new(Span::styled("  no changes", Style::default().fg(C_DIM)))]
    } else {
        app.log.commit_files.iter().map(|f| {
            let (status_str, status_color) = match f.status {
                'A' => ("+ ", C_GREEN),
                'D' => ("- ", C_RED),
                'R' => ("→ ", C_CYAN),
                _   => ("~ ", C_YELLOW),
            };
            ListItem::new(Line::from(vec![
                Span::styled(status_str, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::styled(file_basename(&f.path), Style::default().fg(C_WHITE)),
                Span::styled(file_dir(&f.path), Style::default().fg(C_DIM)),
            ]))
        }).collect()
    };

    let commit_info = app.commits.get(app.log.idx).map(|c| {
        format!(" {} — {} files ", &c.hash, app.log.commit_files.len())
    }).unwrap_or_default();

    let files_block = Block::default()
        .title(Span::styled(commit_info, Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));
    f.render_widget(List::new(file_items).block(files_block), chunks[1]);
}

fn file_basename(path: &str) -> String {
    path.rfind('/').map(|i| path[i+1..].to_string()).unwrap_or_else(|| path.to_string())
}

fn file_dir(path: &str) -> String {
    match path.rfind('/') {
        Some(i) => format!("  {}/", &path[..i]),
        None    => String::new(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 { return String::new(); }
    if s.chars().count() <= max { return s.to_string(); }
    let cut: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", cut)
}
