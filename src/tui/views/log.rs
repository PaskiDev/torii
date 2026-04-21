use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::{App, LogConfirm};
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_CYAN, C_YELLOW, C_GREEN};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let bc = app.brand_color();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    let inner_width = chunks[0].width.saturating_sub(4) as usize;
    let msg_width = inner_width.saturating_sub(32);

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

    let list_block = Block::default()
        .title(Span::styled(
            format!(" log — {} ({} commits) ", app.branch, app.commits.len()),
            Style::default().fg(C_SUBTLE),
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));
    f.render_stateful_widget(List::new(items).block(list_block), chunks[0], &mut state);

    // ── Bottom bar: hints / confirm / input ───────────────────────────────────
    let (bottom_content, bottom_border_color) = match &app.log.confirm {
        LogConfirm::ResetSoft => {
            let hash = app.commits.get(app.log.idx).map(|c| c.hash.as_str()).unwrap_or("?");
            let line = Line::from(vec![
                Span::raw(" "),
                Span::styled("reset --soft ", Style::default().fg(C_SUBTLE)),
                Span::styled(hash, Style::default().fg(C_YELLOW).add_modifier(Modifier::BOLD)),
                Span::styled("  confirm? ", Style::default().fg(C_SUBTLE)),
                Span::styled("[y]", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled(" yes  ", Style::default().fg(C_DIM)),
                Span::styled("[any]", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled(" cancel ", Style::default().fg(C_DIM)),
            ]);
            (line, C_YELLOW)
        }
        LogConfirm::NewBranch => {
            let hash = app.commits.get(app.log.idx).map(|c| c.hash.as_str()).unwrap_or("?");
            let line = Line::from(vec![
                Span::raw(" "),
                Span::styled("new branch at ", Style::default().fg(C_SUBTLE)),
                Span::styled(hash, Style::default().fg(C_YELLOW)),
                Span::styled("  name: ", Style::default().fg(C_SUBTLE)),
                Span::styled(app.log.branch_input.as_str(), Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(bc)),
            ]);
            (line, C_WHITE)
        }
        LogConfirm::None => {
            let status_span = if let Some(s) = &app.log.status {
                Span::styled(format!(" {}  ", s), Style::default().fg(C_GREEN))
            } else {
                Span::raw("")
            };
            let line = Line::from(vec![
                Span::raw(" "),
                Span::styled("[Enter]", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled(" diff  ", Style::default().fg(C_DIM)),
                Span::styled("[r]", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled(" reset soft  ", Style::default().fg(C_DIM)),
                Span::styled("[b]", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled(" new branch ", Style::default().fg(C_DIM)),
                status_span,
            ]);
            (line, bc)
        }
    };

    let bottom_block = Block::default()
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bottom_border_color));
    f.render_widget(Paragraph::new(bottom_content).block(bottom_block), chunks[1]);
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    let cut: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", cut)
}
