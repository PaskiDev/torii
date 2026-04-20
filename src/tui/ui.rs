use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Padding, Paragraph},
};

use super::app::{App, View};
use super::views;

pub const BRAND_COLOR: Color = Color::Rgb(255, 76, 76);
pub const SELECTED_BG: Color = Color::Rgb(40, 40, 60);

// Paleta viva
pub const C_WHITE: Color    = Color::Rgb(220, 220, 220);
pub const C_SUBTLE: Color   = Color::Rgb(140, 140, 160);
pub const C_DIM: Color      = Color::Rgb(80, 80, 100);
pub const C_CYAN: Color     = Color::Rgb(80, 220, 200);
pub const C_YELLOW: Color   = Color::Rgb(255, 210, 80);
pub const C_GREEN: Color    = Color::Rgb(100, 220, 100);
pub const C_RED: Color      = Color::Rgb(255, 100, 100);
pub const C_BORDER: Color   = Color::Rgb(60, 60, 80);

const SIDEBAR_WIDTH: u16 = 20;

struct Tab {
    key: &'static str,
    label: &'static str,
    view: View,
}

const TABS: &[Tab] = &[
    Tab { key: "f", label: "files",     view: View::Dashboard  },
    Tab { key: "c", label: "save",      view: View::Commit     },
    Tab { key: "s", label: "sync",      view: View::Sync       },
    Tab { key: "p", label: "snapshot",  view: View::Snapshot   },
    Tab { key: "l", label: "log",       view: View::Log        },
    Tab { key: "b", label: "branch",    view: View::Branch     },
    Tab { key: "t", label: "tags",      view: View::Tag        },
    Tab { key: "h", label: "history",   view: View::History    },
    Tab { key: "r", label: "remote",    view: View::Remote     },
    Tab { key: "m", label: "mirror",    view: View::Mirror     },
    Tab { key: "w", label: "workspace", view: View::Workspace  },
];

pub fn render(f: &mut Frame, app: &App) {
    if app.view == View::Diff || app.view == View::Help {
        match app.view {
            View::Diff => views::diff::render(f, app),
            View::Help => views::help::render(f, app),
            _          => {}
        }
        return;
    }

    let area = f.area();

    // Global layout: sidebar | content / hint
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(SIDEBAR_WIDTH),
            Constraint::Min(1),
        ])
        .split(area);

    // Content area: main view + 1 line hint at bottom
    let content_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(cols[1]);

    render_sidebar(f, app, cols[0]);

    match app.view {
        View::Dashboard => views::dashboard::render(f, app, content_rows[0]),
        View::Commit    => views::commit::render(f, app, content_rows[0]),
        View::Sync      => views::sync::render(f, app, content_rows[0]),
        View::Snapshot  => views::snapshot::render(f, app, content_rows[0]),
        View::Log       => views::log::render(f, app, content_rows[0]),
        View::Branch    => views::branch::render(f, app, content_rows[0]),
        View::Tag       => views::tag::render(f, app, content_rows[0]),
        View::History   => views::history::render(f, app, content_rows[0]),
        View::Remote    => views::remote::render(f, app, content_rows[0]),
        View::Mirror    => views::mirror::render(f, app, content_rows[0]),
        View::Workspace => views::workspace::render(f, app, content_rows[0]),
        View::Diff | View::Help => {}
    }

    render_hint(f, app, content_rows[1]);
}

fn render_hint(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    if app.sidebar_focused {
        let line = Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" open  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Esc]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" cancel", Style::default().fg(C_SUBTLE)),
        ]);
        f.render_widget(Paragraph::new(line), area);
        return;
    }
    let line = match app.view {
        View::Dashboard => {
            use crate::tui::app::Panel;
            match app.dashboard.selected_panel {
                Panel::Staged => Line::from(vec![
                    Span::raw(" "),
                    Span::styled("[space]", Style::default().fg(BRAND_COLOR)),
                    Span::styled(" unstage  ", Style::default().fg(C_SUBTLE)),
                    Span::styled("[d]", Style::default().fg(BRAND_COLOR)),
                    Span::styled(" diff", Style::default().fg(C_SUBTLE)),
                ]),
                Panel::Unstaged => Line::from(vec![
                    Span::raw(" "),
                    Span::styled("[space]", Style::default().fg(BRAND_COLOR)),
                    Span::styled(" stage  ", Style::default().fg(C_SUBTLE)),
                    Span::styled("[d]", Style::default().fg(BRAND_COLOR)),
                    Span::styled(" diff", Style::default().fg(C_SUBTLE)),
                ]),
                Panel::Untracked => Line::from(vec![
                    Span::raw(" "),
                    Span::styled("[space]", Style::default().fg(BRAND_COLOR)),
                    Span::styled(" stage", Style::default().fg(C_SUBTLE)),
                ]),
                Panel::Log => Line::from(vec![
                    Span::raw(" "),
                    Span::styled("[d]", Style::default().fg(BRAND_COLOR)),
                    Span::styled(" diff  ", Style::default().fg(C_SUBTLE)),
                    Span::styled("[l]", Style::default().fg(BRAND_COLOR)),
                    Span::styled(" expand", Style::default().fg(C_SUBTLE)),
                ]),
            }
        }
        View::Commit => Line::from(vec![
            Span::raw(" "),
            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" save  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[←→]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" cursor  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Esc]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" cancel", Style::default().fg(C_SUBTLE)),
        ]),
        View::Sync => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" select  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" run  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Esc]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" cancel", Style::default().fg(C_SUBTLE)),
        ]),
        View::Log => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[d]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" diff", Style::default().fg(C_SUBTLE)),
        ]),
        View::Branch => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" checkout", Style::default().fg(C_SUBTLE)),
        ]),
        View::Snapshot => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" restore", Style::default().fg(C_SUBTLE)),
        ]),
        View::Tag => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" push  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[d]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" delete", Style::default().fg(C_SUBTLE)),
        ]),
        View::History => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" cherry-pick", Style::default().fg(C_SUBTLE)),
        ]),
        View::Remote => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" info", Style::default().fg(C_SUBTLE)),
        ]),
        View::Mirror => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" sync", Style::default().fg(C_SUBTLE)),
        ]),
        View::Workspace => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
            Span::styled(" sync all", Style::default().fg(C_SUBTLE)),
        ]),
        _ => Line::from(""),
    };
    f.render_widget(Paragraph::new(line), area);
}

fn render_sidebar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let border_color = if app.sidebar_focused { BRAND_COLOR } else { C_BORDER };

    // Single right border for the whole sidebar column
    let outer = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(border_color));
    let inner_area = outer.inner(area);
    f.render_widget(outer, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // brand: name + branch + status
            Constraint::Min(1),     // tabs
            Constraint::Length(2),  // help + quit
        ])
        .split(inner_area);

    // Brand
    let (status_label, status_color) = if app.ahead > 0 && app.behind > 0 {
        (format!("↑{} ↓{}", app.ahead, app.behind), C_YELLOW)
    } else if app.ahead > 0 {
        (format!("↑{} ahead", app.ahead), C_CYAN)
    } else if app.behind > 0 {
        (format!("↓{} behind", app.behind), C_RED)
    } else {
        ("synced".to_string(), C_GREEN)
    };

    let brand = Paragraph::new(vec![
        Line::from(Span::styled("⛩  gitorii", Style::default().fg(BRAND_COLOR).add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("branch: ", Style::default().fg(C_SUBTLE)),
            Span::styled(&app.branch, Style::default().fg(C_GREEN).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("status: ", Style::default().fg(C_SUBTLE)),
            Span::styled(status_label, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]),
    ])
    .block(Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(border_color))
        .padding(Padding::new(1, 1, 0, 0)));
    f.render_widget(brand, rows[0]);

    // Tabs
    let tab_items: Vec<ListItem> = TABS.iter().enumerate().map(|(i, tab)| {
        let is_current_view = app.view == tab.view;
        let is_sidebar_sel  = app.sidebar_focused && i == app.sidebar_idx;

        let (prefix, label_style, bg) = if is_current_view {
            ("█ ", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD), Some(SELECTED_BG))
        } else if is_sidebar_sel {
            ("▶ ", Style::default().fg(C_WHITE), Some(SELECTED_BG))
        } else {
            ("  ", Style::default().fg(C_SUBTLE), None)
        };

        let accent = if is_current_view || is_sidebar_sel { BRAND_COLOR } else { BRAND_COLOR };
        let mut item = ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(BRAND_COLOR)),
            Span::styled(format!("{:<9}", tab.label), label_style),
            Span::styled("[", Style::default().fg(accent)),
            Span::styled(tab.key, Style::default().fg(accent).add_modifier(if is_current_view { Modifier::BOLD } else { Modifier::empty() })),
            Span::styled("]", Style::default().fg(accent)),
        ]));
        if let Some(color) = bg {
            item = item.style(Style::default().bg(color));
        }
        item
    }).collect();

    f.render_widget(
        List::new(tab_items)
            .block(Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(border_color))
                .padding(Padding::new(1, 1, 0, 0))),
        rows[1],
    );

    // Help + quit — aligned with hint row at bottom
    let bottom = List::new(vec![
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<9}", "help"), Style::default().fg(C_SUBTLE)),
            Span::styled("[", Style::default().fg(BRAND_COLOR)),
            Span::styled("?", Style::default().fg(BRAND_COLOR)),
            Span::styled("]", Style::default().fg(BRAND_COLOR)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{:<9}", "quit"), Style::default().fg(C_SUBTLE)),
            Span::styled("[", Style::default().fg(BRAND_COLOR)),
            Span::styled("q", Style::default().fg(BRAND_COLOR)),
            Span::styled("]", Style::default().fg(BRAND_COLOR)),
        ])),
    ]);
    f.render_widget(
        bottom.block(Block::default().borders(Borders::NONE)),
        rows[2],
    );
}
