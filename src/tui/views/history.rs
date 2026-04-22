use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::{App, HistoryConfirm};
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_YELLOW, C_RED};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let bc = app.brand_color();
    let focused = !app.sidebar_focused;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);

    // ── Reflog list ───────────────────────────────────────────────────────────
    let inner_width = chunks[0].width.saturating_sub(4) as usize;
    let msg_width = inner_width.saturating_sub(20);

    let items: Vec<ListItem> = if app.history_view.reflog.is_empty() {
        vec![ListItem::new(Span::styled("  no reflog entries", Style::default().fg(C_DIM)))]
    } else {
        app.history_view.reflog.iter().enumerate().map(|(i, e)| {
            let is_sel = i == app.history_view.idx;
            let style = if is_sel {
                Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if is_sel { "█ " } else { "  " };
            let msg = truncate(&e.message, msg_width);
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(bc)),
                Span::styled(format!("{} ", &e.id), Style::default().fg(C_YELLOW)),
                Span::styled(format!("{:<width$}", msg, width = msg_width), Style::default().fg(C_WHITE)),
                Span::styled(&e.time, Style::default().fg(C_DIM)),
            ])).style(style)
        }).collect()
    };

    let mut state = ListState::default();
    if !app.history_view.reflog.is_empty() { state.select(Some(app.history_view.idx)); }

    let list_block = Block::default()
        .title(Span::styled(
            format!(" history — {} entries ", app.history_view.reflog.len()),
            if focused { Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD) }
            else { Style::default().fg(C_SUBTLE) },
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(if focused { Style::default().fg(C_WHITE) } else { Style::default().fg(bc) });
    f.render_stateful_widget(List::new(items).block(list_block), chunks[0], &mut state);

    // ── Right panel: entry detail or operation feedback ───────────────────────
    let right_lines = match &app.history_view.confirm {
        HistoryConfirm::Scan => {
            let mode = if app.history_view.scan_full { "full history" } else { "staged" };
            vec![
                Line::from(Span::styled("  scan for secrets", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  mode   ", Style::default().fg(C_SUBTLE)),
                    Span::styled(mode, Style::default().fg(C_YELLOW).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  [f]", Style::default().fg(bc)),
                    Span::styled(" toggle mode  ", Style::default().fg(C_SUBTLE)),
                    Span::styled("[Enter]", Style::default().fg(bc)),
                    Span::styled(" run", Style::default().fg(C_SUBTLE)),
                ]),
            ]
        }
        HistoryConfirm::Blame => {
            if !app.history_view.input.is_empty() && app.history_view.input.contains('\n') {
                app.history_view.input.lines().take(20).map(|l| {
                    Line::from(Span::styled(format!("  {}", l), Style::default().fg(C_WHITE)))
                }).collect()
            } else {
                vec![
                    Line::from(Span::styled("  blame", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(Span::styled("  enter file path", Style::default().fg(C_DIM))),
                ]
            }
        }
        _ => {
            if let Some(e) = app.history_view.reflog.get(app.history_view.idx) {
                vec![
                    Line::from(vec![
                        Span::styled("  hash     ", Style::default().fg(C_SUBTLE)),
                        Span::styled(&e.id, Style::default().fg(C_YELLOW).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled("  message  ", Style::default().fg(C_SUBTLE)),
                        Span::styled(&e.message, Style::default().fg(C_WHITE)),
                    ]),
                    Line::from(vec![
                        Span::styled("  age      ", Style::default().fg(C_SUBTLE)),
                        Span::styled(&e.time, Style::default().fg(C_DIM)),
                    ]),
                ]
            } else {
                vec![Line::from(Span::styled("  no entry selected", Style::default().fg(C_DIM)))]
            }
        }
    };

    let right_title = match &app.history_view.confirm {
        HistoryConfirm::None => " detail ",
        HistoryConfirm::Scan => " scan ",
        HistoryConfirm::Blame => " blame ",
        _ => " detail ",
    };

    let right_block = Block::default()
        .title(Span::styled(right_title, Style::default().fg(C_SUBTLE)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));
    f.render_widget(Paragraph::new(right_lines).block(right_block), chunks[1]);

    // ── Ops dropdown overlay ──────────────────────────────────────────────────
    if app.history_view.ops_mode {
        const OPS: &[(&str, bool)] = &[
            ("cherry-pick",   false),
            ("rebase onto…",  false),
            ("scan secrets",  false),
            ("clean history", false),
            ("blame file…",   false),
            ("rewrite dates", false),
            ("remove file ⚠", true),
        ];
        let dropdown_w = 22u16;
        let dropdown_h = OPS.len() as u16 + 2;

        // Position: left edge of list panel + 2, just below the selected entry
        // header(1 border) + idx row + 1 border top = idx + 2
        let entry_y = chunks[0].y + 1 + app.history_view.idx as u16 + 1;
        let drop_y = if entry_y + dropdown_h < chunks[0].y + chunks[0].height {
            entry_y
        } else {
            chunks[0].y + chunks[0].height - dropdown_h
        };
        let drop_x = chunks[0].x + 3;
        let drop_area = Rect::new(drop_x, drop_y, dropdown_w, dropdown_h);

        let items: Vec<ListItem> = OPS.iter().enumerate().map(|(i, (label, danger))| {
            let is_sel = i == app.history_view.ops_idx;
            let color = if *danger { C_RED } else if is_sel { C_WHITE } else { C_SUBTLE };
            let prefix = if is_sel { "▶ " } else { "  " };
            let style = if is_sel {
                Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(bc)),
                Span::styled(*label, Style::default().fg(color)),
            ])).style(style)
        }).collect();

        let mut drop_state = ListState::default();
        drop_state.select(Some(app.history_view.ops_idx));

        let drop_block = Block::default()
            .borders(Borders::ALL).border_type(app.border_type())
            .border_style(Style::default().fg(bc));

        f.render_widget(Clear, drop_area);
        f.render_stateful_widget(List::new(items).block(drop_block), drop_area, &mut drop_state);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max { return s.to_string(); }
    let cut: String = s.chars().take(max.saturating_sub(1)).collect();
    format!("{}…", cut)
}
