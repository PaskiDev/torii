use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use super::app::{App, Panel};

const BRAND_COLOR: Color = Color::Rgb(255, 140, 0); // orange
const SELECTED_BG: Color = Color::Rgb(40, 40, 60);

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    // Layout: header | files (3 cols) | log | footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // header
            Constraint::Length(12), // file panels
            Constraint::Min(6),     // log
            Constraint::Length(1),  // footer
        ])
        .split(area);

    render_header(f, app, chunks[0]);
    render_files(f, app, chunks[1]);
    render_log(f, app, chunks[2]);
    render_footer(f, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let ahead_behind = if app.ahead > 0 || app.behind > 0 {
        format!("  ↑{} ↓{}", app.ahead, app.behind)
    } else {
        "  ✓ up to date".to_string()
    };

    let text = Line::from(vec![
        Span::styled(" ⛩  gitorii", Style::default().fg(BRAND_COLOR).add_modifier(Modifier::BOLD)),
        Span::raw("  │  "),
        Span::styled(" ", Style::default().fg(Color::Cyan)),
        Span::styled(&app.branch, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(ahead_behind, Style::default().fg(Color::DarkGray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let para = Paragraph::new(text).block(block);
    f.render_widget(para, area);
}

fn render_files(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(area);

    render_file_list(f, app, cols[0], Panel::Staged,    &app.staged,    app.staged_idx,    "Staged");
    render_file_list(f, app, cols[1], Panel::Unstaged,  &app.unstaged,  app.unstaged_idx,  "Unstaged");
    render_file_list(f, app, cols[2], Panel::Untracked, &app.untracked, app.untracked_idx, "Untracked");
}

fn render_file_list(
    f: &mut Frame,
    app: &App,
    area: Rect,
    panel: Panel,
    files: &[super::app::FileEntry],
    selected: usize,
    title: &str,
) {
    let is_active = app.selected_panel == panel;
    let border_style = if is_active {
        Style::default().fg(BRAND_COLOR)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title_style = if is_active {
        Style::default().fg(BRAND_COLOR).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let count = files.len();
    let title_str = format!(" {} ({}) ", title, count);

    let block = Block::default()
        .title(Span::styled(title_str, title_style))
        .borders(Borders::ALL)
        .border_style(border_style);

    let items: Vec<ListItem> = files.iter().enumerate().map(|(i, f)| {
        let style = if is_active && i == selected {
            Style::default().bg(SELECTED_BG).fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let prefix = if is_active && i == selected { "▶ " } else { "  " };
        ListItem::new(format!("{}{}", prefix, shorten_path(&f.path, area.width as usize - 4))).style(style)
    }).collect();

    let mut state = ListState::default();
    if is_active && !files.is_empty() {
        state.select(Some(selected));
    }

    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

fn render_log(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.selected_panel == Panel::Log;
    let border_style = if is_active {
        Style::default().fg(BRAND_COLOR)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let title_style = if is_active {
        Style::default().fg(BRAND_COLOR).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .title(Span::styled(" Log ", title_style))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_width = area.width.saturating_sub(4) as usize;
    let msg_width = inner_width.saturating_sub(22);

    let items: Vec<ListItem> = app.commits.iter().enumerate().map(|(i, c)| {
        let is_sel = is_active && i == app.log_idx;
        let style = if is_sel {
            Style::default().bg(SELECTED_BG).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let prefix = if is_sel { "▶ " } else { "  " };
        let msg = truncate(&c.message, msg_width);
        let line = Line::from(vec![
            Span::raw(prefix),
            Span::styled(format!("{} ", c.hash), Style::default().fg(Color::Yellow)),
            Span::styled(format!("{:<width$}", msg, width = msg_width), Style::default().fg(Color::White)),
            Span::styled(format!(" {}", c.time), Style::default().fg(Color::DarkGray)),
        ]);
        ListItem::new(line).style(style)
    }).collect();

    let mut state = ListState::default();
    if is_active && !app.commits.is_empty() {
        state.select(Some(app.log_idx));
    }

    f.render_stateful_widget(List::new(items).block(block), area, &mut state);
}

fn render_footer(f: &mut Frame, area: Rect) {
    let spans = Line::from(vec![
        Span::styled(" [Tab]", Style::default().fg(BRAND_COLOR)),
        Span::raw(" panel  "),
        Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
        Span::raw(" navigate  "),
        Span::styled("[r]", Style::default().fg(BRAND_COLOR)),
        Span::raw(" refresh  "),
        Span::styled("[q]", Style::default().fg(BRAND_COLOR)),
        Span::raw(" quit"),
    ]);
    f.render_widget(Paragraph::new(spans), area);
}

fn shorten_path(path: &str, max: usize) -> String {
    if path.len() <= max { return path.to_string(); }
    format!("…{}", &path[path.len().saturating_sub(max - 1)..])
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { return s.to_string(); }
    format!("{}…", &s[..max.saturating_sub(1)])
}
