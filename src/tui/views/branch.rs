use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::{App, BranchConfirm};
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_GREEN, C_RED, C_YELLOW};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let bc = app.brand_color();
    let focused = !app.sidebar_focused;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    // ── Branch list ───────────────────────────────────────────────────────────
    let locals: Vec<(usize, &crate::tui::app::BranchEntry)> = app.branch_view.branches
        .iter().enumerate().filter(|(_, b)| !b.is_remote).collect();
    let remotes: Vec<(usize, &crate::tui::app::BranchEntry)> = app.branch_view.branches
        .iter().enumerate().filter(|(_, b)| b.is_remote).collect();

    let mut items: Vec<ListItem> = vec![];

    if !locals.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::styled(" local ", Style::default().fg(C_SUBTLE).add_modifier(Modifier::BOLD)),
        ])));
        for (i, b) in &locals {
            items.push(branch_item(app, *i, b, bc));
        }
    }

    if !remotes.is_empty() {
        items.push(ListItem::new(Line::from(vec![
            Span::raw(" "),
        ])));
        items.push(ListItem::new(Line::from(vec![
            Span::styled(" remote ", Style::default().fg(C_SUBTLE).add_modifier(Modifier::BOLD)),
        ])));
        for (i, b) in &remotes {
            items.push(branch_item(app, *i, b, bc));
        }
    }

    // Map logical idx to list position (account for header rows)
    let sel_list_pos = {
        let idx = app.branch_view.idx;
        let is_remote = app.branch_view.branches.get(idx).map(|b| b.is_remote).unwrap_or(false);
        if !is_remote {
            let pos_in_locals = locals.iter().position(|(i, _)| *i == idx).unwrap_or(0);
            1 + pos_in_locals // +1 for "local" header
        } else {
            let pos_in_remotes = remotes.iter().position(|(i, _)| *i == idx).unwrap_or(0);
            locals.len() + 3 + pos_in_remotes // locals header + locals + blank + remotes header
        }
    };

    let local_count = locals.len();
    let remote_count = remotes.len();
    let title = format!(" branches — {} local  {} remote ", local_count, remote_count);

    let mut state = ListState::default();
    if !app.branch_view.branches.is_empty() { state.select(Some(sel_list_pos)); }

    let list_block = Block::default()
        .title(Span::styled(title,
            if focused { Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD) }
            else { Style::default().fg(C_SUBTLE) }
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(if focused { Style::default().fg(C_WHITE) } else { Style::default().fg(bc) });
    f.render_stateful_widget(List::new(items).block(list_block), chunks[0], &mut state);

    // ── Bottom bar: status / confirm / input ──────────────────────────────────
    let (bottom_line, border_color) = match &app.branch_view.confirm {
        BranchConfirm::Delete => {
            let name = app.branch_view.branches.get(app.branch_view.idx)
                .map(|b| b.name.as_str()).unwrap_or("?");
            (Line::from(vec![
                Span::raw(" "),
                Span::styled("delete ", Style::default().fg(C_SUBTLE)),
                Span::styled(name, Style::default().fg(C_RED).add_modifier(Modifier::BOLD)),
                Span::styled("?  ", Style::default().fg(C_SUBTLE)),
                Span::styled("[y]", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled(" confirm  ", Style::default().fg(C_DIM)),
                Span::styled("[any]", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled(" cancel ", Style::default().fg(C_DIM)),
            ]), C_RED)
        }
        BranchConfirm::NewBranch => {
            (Line::from(vec![
                Span::raw(" "),
                Span::styled("new branch: ", Style::default().fg(C_SUBTLE)),
                Span::styled(app.branch_view.new_name.as_str(),
                    Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(bc)),
            ]), C_WHITE)
        }
        BranchConfirm::None => {
            let content = if let Some(s) = &app.branch_view.status {
                let color = if s.starts_with("checkout:") || s.starts_with("created") || s.starts_with("deleted") {
                    C_GREEN
                } else if s.contains("failed") || s.contains("cannot") {
                    C_RED
                } else {
                    C_YELLOW
                };
                Line::from(vec![Span::raw(" "), Span::styled(s.as_str(), Style::default().fg(color))])
            } else {
                Line::from(vec![
                    Span::raw(" "),
                    Span::styled("[Enter]", Style::default().fg(bc)),
                    Span::styled(" checkout  ", Style::default().fg(C_DIM)),
                    Span::styled("[n]", Style::default().fg(bc)),
                    Span::styled(" new  ", Style::default().fg(C_DIM)),
                    Span::styled("[d]", Style::default().fg(bc)),
                    Span::styled(" delete ", Style::default().fg(C_DIM)),
                ])
            };
            (content, bc)
        }
    };

    let bottom_block = Block::default()
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(border_color));
    f.render_widget(Paragraph::new(bottom_line).block(bottom_block), chunks[1]);
}

fn branch_item<'a>(
    app: &App,
    idx: usize,
    b: &'a crate::tui::app::BranchEntry,
    bc: ratatui::style::Color,
) -> ListItem<'a> {
    let is_sel = idx == app.branch_view.idx;
    let style = if is_sel {
        Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let prefix = if is_sel { "█ " } else { "  " };
    let current = if b.is_current { "* " } else { "  " };
    let name_color = if b.is_current { C_GREEN } else if is_sel { C_WHITE } else { C_SUBTLE };

    ListItem::new(Line::from(vec![
        Span::styled(prefix, Style::default().fg(bc)),
        Span::styled(current, Style::default().fg(C_GREEN)),
        Span::styled(b.name.clone(), Style::default().fg(name_color)),
    ])).style(style)
}
