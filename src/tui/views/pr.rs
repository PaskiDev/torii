use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::tui::app::{App, PrConfirm, PrStateFilter};
use super::super::ui::{C_WHITE, C_SUBTLE, C_GREEN, C_RED, C_YELLOW, C_CYAN, C_DIM};

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let bc = app.brand_color();
    let focused = !app.sidebar_focused;
    let pr = &app.pr_view;

    // Split: list (60%) | detail (40%)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    // ── Filter badge ──────────────────────────────────────────────────────────
    let filter_str = match pr.filter {
        PrStateFilter::Open   => "open",
        PrStateFilter::Closed => "closed",
        PrStateFilter::All    => "all",
    };

    // ── PR list ───────────────────────────────────────────────────────────────
    let items: Vec<ListItem> = if pr.loading {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  loading...", Style::default().fg(C_SUBTLE)),
        ]))]
    } else if let Some(err) = &pr.error {
        let is_token = err.to_lowercase().contains("token");
        // Split long error into multiple items for readability
        let mut err_items = vec![ListItem::new(Line::from(vec![
            Span::styled("  ✗ ", Style::default().fg(C_RED)),
            Span::styled(
                if is_token { "authentication required".to_string() } else { "error".to_string() },
                Style::default().fg(C_RED).add_modifier(Modifier::BOLD)
            ),
        ]))];
        for chunk in err.chars().collect::<Vec<_>>().chunks(50) {
            let s: String = chunk.iter().collect();
            err_items.push(ListItem::new(Line::from(vec![
                Span::styled(format!("  {}", s), Style::default().fg(C_SUBTLE)),
            ])));
        }
        err_items
    } else if pr.prs.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("  no pull requests", Style::default().fg(C_DIM)),
        ]))]
    } else {
        pr.prs.iter().enumerate().map(|(i, p)| {
            let is_sel = i == pr.idx;
            let style = if is_sel {
                Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let prefix = if is_sel { "█ " } else { "  " };
            let state_color = if p.state == "open" { C_GREEN } else { C_SUBTLE };
            let draft_tag = if p.draft { " [draft]" } else { "" };
            let num_str = format!("#{}", p.number);
            let title_color = if is_sel { C_WHITE } else { C_SUBTLE };

            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(bc)),
                Span::styled(num_str, Style::default().fg(state_color)),
                Span::styled(" ", Style::default()),
                Span::styled(p.title.clone(), Style::default().fg(title_color)),
                Span::styled(draft_tag, Style::default().fg(C_DIM)),
            ])).style(style)
        }).collect()
    };

    let count = pr.prs.len();
    let pr_label = if pr.platform == "gitlab" { "merge requests" } else { "pull requests" };
    let title = format!(" {} — {} [{}] ", pr_label, count, filter_str);
    let mut list_state = ListState::default();
    if !pr.prs.is_empty() { list_state.select(Some(pr.idx)); }

    let list_block = Block::default()
        .title(Span::styled(title,
            if focused { Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD) }
            else { Style::default().fg(bc) }
        ))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(if focused { Style::default().fg(C_WHITE) } else { Style::default().fg(bc) });
    f.render_stateful_widget(List::new(items).block(list_block), cols[0], &mut list_state);

    // ── Detail panel ──────────────────────────────────────────────────────────
    let detail_block = Block::default()
        .title(Span::styled(" detail ", Style::default().fg(bc)))
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(bc));

    if let Some(p) = pr.prs.get(pr.idx) {
        let state_color = if p.state == "open" { C_GREEN } else { C_SUBTLE };
        let mergeable_str = match p.mergeable {
            Some(true)  => Span::styled("✓ mergeable", Style::default().fg(C_GREEN)),
            Some(false) => Span::styled("✗ conflicts", Style::default().fg(C_RED)),
            None        => Span::styled("~ unknown",   Style::default().fg(C_DIM)),
        };

        let mut lines = vec![
            Line::from(vec![
                Span::styled("  #", Style::default().fg(C_SUBTLE)),
                Span::styled(p.number.to_string(), Style::default().fg(state_color).add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(p.state.clone(), Style::default().fg(state_color)),
                if p.draft { Span::styled("  draft", Style::default().fg(C_DIM)) } else { Span::raw("") },
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(p.title.clone(), Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  by  ", Style::default().fg(C_DIM)),
                Span::styled(p.author.clone(), Style::default().fg(C_CYAN)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default().fg(C_DIM)),
                Span::styled(p.head.clone(), Style::default().fg(C_YELLOW)),
                Span::styled(" → ", Style::default().fg(C_DIM)),
                Span::styled(p.base.clone(), Style::default().fg(C_SUBTLE)),
            ]),
            Line::from(vec![
                Span::styled("  ", Style::default()),
                mergeable_str,
            ]),
            Line::from(vec![
                Span::styled("  created  ", Style::default().fg(C_DIM)),
                Span::styled(p.created_at.clone(), Style::default().fg(C_SUBTLE)),
            ]),
        ];

        if let Some(body) = &p.body {
            if !body.trim().is_empty() {
                lines.push(Line::from(""));
                lines.push(Line::from(vec![
                    Span::styled("  ─── description ───", Style::default().fg(C_DIM)),
                ]));
                for l in body.lines().take(12) {
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {}", l), Style::default().fg(C_SUBTLE)),
                    ]));
                }
            }
        }

        let para = Paragraph::new(lines)
            .block(detail_block)
            .wrap(Wrap { trim: false });
        f.render_widget(para, cols[1]);
    } else {
        let para = Paragraph::new(Line::from(vec![
            Span::styled("  select a PR", Style::default().fg(C_DIM)),
        ])).block(detail_block);
        f.render_widget(para, cols[1]);
    }

    // ── Confirm overlay ───────────────────────────────────────────────────────
    if pr.confirm == PrConfirm::Close {
        let overlay = Rect::new(
            cols[0].x + 2,
            cols[0].y + 2 + pr.idx.min(cols[0].height as usize - 5) as u16,
            28, 3,
        );
        f.render_widget(Clear, overlay);
        let p = Paragraph::new(Line::from(vec![
            Span::styled("  close PR? ", Style::default().fg(C_RED)),
            Span::styled("[y]", Style::default().fg(C_RED).add_modifier(Modifier::BOLD)),
            Span::styled(" / any", Style::default().fg(C_SUBTLE)),
        ])).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(C_RED)).border_type(app.border_type()));
        f.render_widget(p, overlay);
    }

    if pr.confirm == PrConfirm::Merge {
        let methods = ["merge", "squash", "rebase"];
        let head_branch = pr.prs.get(pr.idx).map(|p| p.head.as_str()).unwrap_or("?");
        let overlay = Rect::new(
            cols[0].x + 2,
            cols[0].y + 2 + pr.idx.min(cols[0].height as usize - 8) as u16,
            34, 6,
        );
        f.render_widget(Clear, overlay);
        let method_spans: Vec<Span> = methods.iter().enumerate().map(|(i, m)| {
            if i == pr.merge_method {
                Span::styled(format!(" [{}] ", m), Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD).bg(app.selected_bg()))
            } else {
                Span::styled(format!("  {}  ", m), Style::default().fg(C_SUBTLE))
            }
        }).collect();
        let lines = vec![
            Line::from(vec![Span::styled("  merge method:", Style::default().fg(C_SUBTLE))]),
            Line::from(method_spans),
            Line::from(vec![
                Span::styled("  branch '", Style::default().fg(C_SUBTLE)),
                Span::styled(head_branch.to_string(), Style::default().fg(C_YELLOW)),
                Span::styled("' will be deleted", Style::default().fg(C_SUBTLE)),
            ]),
            Line::from(vec![
                Span::styled("  [←→]", Style::default().fg(bc)),
                Span::styled(" select  ", Style::default().fg(C_SUBTLE)),
                Span::styled("[Enter]", Style::default().fg(bc)),
                Span::styled(" confirm", Style::default().fg(C_SUBTLE)),
            ]),
        ];
        let p = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(bc)).border_type(app.border_type()));
        f.render_widget(p, overlay);
    }

    // ── Create overlays ───────────────────────────────────────────────────────
    let pr_label = if pr.platform == "gitlab" { "MR" } else { "PR" };
    if matches!(pr.confirm, PrConfirm::CreateTitle | PrConfirm::CreateBase) {
        let (step, label) = match &pr.confirm {
            PrConfirm::CreateTitle => (1, "title"),
            PrConfirm::CreateBase  => (2, "base branch"),
            _ => (0, ""),
        };
        let ow = 52u16;
        let oh = 5u16;
        let ox = area.x + area.width.saturating_sub(ow) / 2;
        let oy = area.y + area.height.saturating_sub(oh) / 2;
        let overlay = Rect::new(ox, oy, ow, oh);
        let cursor = format!("{}█", pr.create_input);
        let lines = vec![
            Line::from(vec![
                Span::styled(format!("  create {} — step {}/3: ", pr_label, step), Style::default().fg(C_SUBTLE)),
                Span::styled(label, Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("  > {}", cursor), Style::default().fg(C_CYAN)),
            ]),
            Line::from(vec![
                Span::styled("  [Enter]", Style::default().fg(bc)),
                Span::styled(" next  ", Style::default().fg(C_SUBTLE)),
                Span::styled("[Esc]", Style::default().fg(bc)),
                Span::styled(" cancel", Style::default().fg(C_SUBTLE)),
            ]),
        ];
        f.render_widget(Clear, overlay);
        f.render_widget(
            Paragraph::new(lines).block(
                Block::default().borders(Borders::ALL)
                    .border_style(Style::default().fg(bc))
                    .border_type(app.border_type())
            ),
            overlay,
        );
    }

    if pr.confirm == PrConfirm::CreateDesc {
        let ow = 74u16;
        let oh = 14u16;
        // centre within the content area (excludes sidebar)
        let ox = area.x + area.width.saturating_sub(ow) / 2;
        let oy = area.y + area.height.saturating_sub(oh) / 2;
        let overlay = Rect::new(ox, oy, ow, oh);

        let draft_hint = if pr.create_draft { "  [draft ✓]" } else { "  [Tab] draft" };
        let mut lines = vec![
            Line::from(vec![
                Span::styled(format!("  create {} — step 3/3: ", pr_label), Style::default().fg(C_SUBTLE)),
                Span::styled("description", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
                Span::styled(" (optional)", Style::default().fg(C_DIM)),
            ]),
        ];
        // accumulated lines
        for l in pr.create_desc.lines() {
            lines.push(Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(l.to_string(), Style::default().fg(C_SUBTLE)),
            ]));
        }
        // current input line with cursor
        lines.push(Line::from(vec![
            Span::styled(format!("  {}█", pr.create_input), Style::default().fg(C_CYAN)),
        ]));
        // fill remaining space
        while lines.len() < (oh as usize - 3) {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(vec![
            Span::styled("  [Enter]", Style::default().fg(bc)),
            Span::styled(" new line  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[c]", Style::default().fg(bc)),
            Span::styled(" create  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Esc]", Style::default().fg(bc)),
            Span::styled(" cancel  ", Style::default().fg(C_SUBTLE)),
            Span::styled(draft_hint, Style::default().fg(C_YELLOW)),
        ]));

        f.render_widget(Clear, overlay);
        f.render_widget(
            Paragraph::new(lines).block(
                Block::default().borders(Borders::ALL)
                    .border_style(Style::default().fg(bc))
                    .border_type(app.border_type())
            ),
            overlay,
        );
    }

    // ── Edit overlays ────────────────────────────────────────────────────────
    let edit_label = if pr.platform == "gitlab" { "MR" } else { "PR" };

    if pr.confirm == PrConfirm::EditTitle {
        let ow = 60u16; let oh = 5u16;
        let ox = area.x + area.width.saturating_sub(ow) / 2;
        let oy = area.y + area.height.saturating_sub(oh) / 2;
        let overlay = Rect::new(ox, oy, ow, oh);
        let lines = vec![
            Line::from(vec![
                Span::styled(format!("  edit {} — step 1/3: ", edit_label), Style::default().fg(C_SUBTLE)),
                Span::styled("title", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("  {}█", pr.edit_input), Style::default().fg(C_CYAN)),
            ]),
            Line::from(vec![
                Span::styled("  [Enter]", Style::default().fg(bc)),
                Span::styled(" next  ", Style::default().fg(C_SUBTLE)),
                Span::styled("[Esc]", Style::default().fg(bc)),
                Span::styled(" cancel", Style::default().fg(C_SUBTLE)),
            ]),
        ];
        f.render_widget(Clear, overlay);
        f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(bc)).border_type(app.border_type())), overlay);
    }

    if pr.confirm == PrConfirm::EditDesc {
        let ow = 74u16; let oh = 14u16;
        let ox = area.x + area.width.saturating_sub(ow) / 2;
        let oy = area.y + area.height.saturating_sub(oh) / 2;
        let overlay = Rect::new(ox, oy, ow, oh);
        let mut lines = vec![
            Line::from(vec![
                Span::styled(format!("  edit {} — step 2/3: ", edit_label), Style::default().fg(C_SUBTLE)),
                Span::styled("description", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD)),
            ]),
        ];
        for l in pr.edit_desc.lines() {
            lines.push(Line::from(vec![Span::raw("  "), Span::styled(l.to_string(), Style::default().fg(C_SUBTLE))]));
        }
        lines.push(Line::from(vec![Span::styled(format!("  █"), Style::default().fg(C_CYAN))]));
        while lines.len() < (oh as usize - 3) { lines.push(Line::from("")); }
        lines.push(Line::from(vec![
            Span::styled("  [Enter]", Style::default().fg(bc)),
            Span::styled(" new line  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[c]", Style::default().fg(bc)),
            Span::styled(" next  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Esc]", Style::default().fg(bc)),
            Span::styled(" cancel", Style::default().fg(C_SUBTLE)),
        ]));
        f.render_widget(Clear, overlay);
        f.render_widget(Paragraph::new(lines).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(bc)).border_type(app.border_type())), overlay);
    }

    if pr.confirm == PrConfirm::EditBase {
        let dw = 30u16;
        let dh = (pr.branches.len().min(10) + 2) as u16;
        let ox = area.x + area.width.saturating_sub(dw) / 2;
        let oy = area.y + area.height.saturating_sub(dh) / 2;
        let drop_area = Rect::new(ox, oy, dw, dh);

        let drop_items: Vec<ListItem> = pr.branches.iter().enumerate().map(|(i, branch)| {
            let is_sel = i == pr.branch_idx;
            let color = if is_sel { C_WHITE } else { C_SUBTLE };
            let prefix = if is_sel { "▶ " } else { "  " };
            ListItem::new(Line::from(vec![
                Span::styled(prefix, Style::default().fg(bc)),
                Span::styled(branch.clone(), Style::default().fg(color)),
            ])).style(if is_sel { Style::default().bg(app.selected_bg()).add_modifier(Modifier::BOLD) } else { Style::default() })
        }).collect();

        let mut drop_state = ListState::default();
        drop_state.select(Some(pr.branch_idx));

        f.render_widget(Clear, drop_area);
        f.render_stateful_widget(
            List::new(drop_items).block(
                Block::default()
                    .title(Span::styled(format!(" step 3/3: base branch "), Style::default().fg(C_SUBTLE)))
                    .borders(Borders::ALL).border_type(app.border_type())
                    .border_style(Style::default().fg(bc))
            ),
            drop_area,
            &mut drop_state,
        );
    }

    // ── Ops dropdown ──────────────────────────────────────────────────────────
    if pr.ops_mode {
        let current_state = pr.prs.get(pr.idx).map(|p| p.state.as_str()).unwrap_or("open");
        let create_label = if pr.platform == "gitlab" { "create MR" } else { "create PR" };
        let ops: &[(&str, bool)] = &[
            (create_label,   false),
            ("edit",         false),
            ("merge",        false),
            ("close ⚠",      true),
            ("checkout",     false),
            ("open browser", false),
        ];

        let dropdown_w = 20u16;
        let dropdown_h = ops.len() as u16 + 2;
        let entry_y = cols[0].y + 1 + pr.idx as u16 + 1;
        let drop_y = if entry_y + dropdown_h < cols[0].y + cols[0].height {
            entry_y
        } else {
            cols[0].y + cols[0].height - dropdown_h
        };
        let drop_area = Rect::new(cols[0].x + 3, drop_y, dropdown_w, dropdown_h);

        let drop_items: Vec<ListItem> = ops.iter().enumerate().map(|(i, (label, danger))| {
            let is_sel = i == pr.ops_idx;
            let dimmed = i == 2 && current_state != "open" && current_state != "opened";
            let color = if dimmed { C_DIM }
                else if *danger { C_RED }
                else if is_sel { C_WHITE }
                else { C_SUBTLE };
            let prefix = if is_sel { "▶ " } else { "  " };
            let style = if is_sel && !dimmed {
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
        drop_state.select(Some(pr.ops_idx));

        let drop_block = Block::default()
            .borders(Borders::ALL).border_type(app.border_type())
            .border_style(Style::default().fg(bc));

        f.render_widget(Clear, drop_area);
        f.render_stateful_widget(List::new(drop_items).block(drop_block), drop_area, &mut drop_state);
    }
}
