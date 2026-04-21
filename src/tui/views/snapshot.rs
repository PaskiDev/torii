use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::tui::app::{App, AutoSnapshotInterval, SnapshotFocus};
use super::super::ui::{C_WHITE, C_SUBTLE, C_DIM, C_YELLOW, C_GREEN};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let in_list   = app.snapshot_view.focus == SnapshotFocus::List;
    let in_create = app.snapshot_view.focus == SnapshotFocus::Create;
    let in_auto   = app.snapshot_view.focus == SnapshotFocus::AutoConfig;

    let bc = app.brand_color();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(AutoSnapshotInterval::all().len() as u16 + 2),
        ])
        .split(area);

    // ── Snapshot list ─────────────────────────────────────────────────────────
    let items: Vec<ListItem> = if app.snapshot_view.snapshots.is_empty() {
        vec![ListItem::new(Span::styled(
            "  no snapshots — press [n] to create one",
            Style::default().fg(C_DIM),
        ))]
    } else {
        app.snapshot_view.snapshots.iter().enumerate().map(|(i, s)| {
            let is_sel = in_list && i == app.snapshot_view.idx;
            ListItem::new(Line::from(vec![
                Span::styled(if is_sel { "▶ " } else { "  " }, Style::default().fg(bc)),
                Span::styled(&s.name, Style::default().fg(if is_sel { C_WHITE } else { C_SUBTLE }).add_modifier(if is_sel { Modifier::BOLD } else { Modifier::empty() })),
                Span::styled(format!("  {}", s.id), Style::default().fg(C_YELLOW)),
                Span::styled(format!("  {}", s.time), Style::default().fg(C_DIM)),
            ])).style(if is_sel { Style::default().bg(app.selected_bg()) } else { Style::default() })
        }).collect()
    };

    let mut state = ListState::default();
    if in_list && !app.snapshot_view.snapshots.is_empty() {
        state.select(Some(app.snapshot_view.idx));
    }

    let list_block = Block::default()
        .title(Span::styled(
            format!(" snapshots ({})  [n] new  [d] delete  [Enter] load  [a] auto ", app.snapshot_view.snapshots.len()),
            if in_list { Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD) } else { Style::default().fg(bc) },
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(if in_list { Style::default().fg(C_WHITE) } else { Style::default().fg(bc) });
    f.render_stateful_widget(List::new(items).block(list_block), chunks[0], &mut state);

    // ── Create input ──────────────────────────────────────────────────────────
    let name = &app.snapshot_view.create_name;
    let input_line = if in_create {
        Line::from(vec![
            Span::raw(" "),
            Span::styled(name.as_str(), Style::default().fg(C_WHITE)),
            Span::styled("█", Style::default().fg(bc)),
        ])
    } else {
        Line::from(vec![
            Span::raw(" "),
            Span::styled(if name.is_empty() { "snapshot name..." } else { name.as_str() },
                Style::default().fg(C_DIM)),
        ])
    };
    let create_block = Block::default()
        .title(Span::styled(
            " new snapshot ",
            if in_create { Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD) } else { Style::default().fg(bc) },
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(if in_create { Style::default().fg(C_WHITE) } else { Style::default().fg(bc) });
    f.render_widget(Paragraph::new(input_line).block(create_block), chunks[1]);

    // ── Auto-snapshot config ──────────────────────────────────────────────────
    let intervals = AutoSnapshotInterval::all();
    let auto_items: Vec<ListItem> = intervals.iter().enumerate().map(|(i, interval)| {
        let is_sel     = in_auto && i == app.snapshot_view.auto_interval_idx;
        let is_current = *interval == app.snapshot_view.auto_interval;
        let indicator  = if is_sel { "▶ " } else { "  " };
        let check      = if is_current { "✓ " } else { "  " };
        ListItem::new(Line::from(vec![
            Span::styled(indicator, Style::default().fg(bc)),
            Span::styled(check, Style::default().fg(C_GREEN)),
            Span::styled(interval.label(), Style::default()
                .fg(if is_sel { C_WHITE } else { C_SUBTLE })
                .add_modifier(if is_sel { Modifier::BOLD } else { Modifier::empty() })),
        ])).style(if is_sel { Style::default().bg(app.selected_bg()) } else { Style::default() })
    }).collect();

    let auto_block = Block::default()
        .title(Span::styled(
            format!(" auto-snapshot  [current: {}] ", app.snapshot_view.auto_interval.label()),
            if in_auto { Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD) } else { Style::default().fg(bc) },
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(if in_auto { Style::default().fg(C_WHITE) } else { Style::default().fg(bc) });
    f.render_widget(List::new(auto_items).block(auto_block), chunks[2]);
}
