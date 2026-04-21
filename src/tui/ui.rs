use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Padding, Paragraph},
};

use super::app::{App, EventKind, View};
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
    Tab { key: "g", label: "config",    view: View::Config     },
    Tab { key: "x", label: "settings",  view: View::Settings   },
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

    // Outer border around the entire UI
    let outer = Block::default()
        .borders(Borders::ALL).border_type(app.border_type())
        .border_style(Style::default().fg(app.brand_color()));
    let inner = outer.inner(area);
    f.render_widget(outer, area);

    // Global layout: sidebar | content
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(SIDEBAR_WIDTH),
            Constraint::Min(1),
        ])
        .split(inner);

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
        View::Config    => views::config::render(f, app, content_rows[0]),
        View::Settings  => views::settings::render(f, app, content_rows[0]),
        View::Diff | View::Help => {}
    }

    render_hint(f, app, content_rows[1]);

    // Intersecciones sidebar ↔ borde exterior — se ponen al final para no ser sobreescritas
    // div_x: columna del divisor RIGHT de la sidebar = inner.x + SIDEBAR_WIDTH - 1
    let div_x = inner.x + SIDEBAR_WIDTH - 1;
    if div_x < area.x + area.width {
        let border_color = if app.sidebar_focused { app.brand_color() } else { C_BORDER };
        let buf = f.buffer_mut();
        let outer_color = app.brand_color();
        buf.cell_mut((div_x, area.y))
            .map(|c| c.set_symbol("┬").set_fg(outer_color));
        buf.cell_mut((div_x, area.y + area.height - 1))
            .map(|c| c.set_symbol("┴").set_fg(outer_color));
        // Intersecciones internas: donde brand y tabs BOTTOM tocan ambos bordes
        let brand_bottom_y = inner.y + 3;
        let tabs_bottom_y  = inner.y + inner.height - 3;
        // Lado derecho (divisor): ┤
        buf.cell_mut((div_x, brand_bottom_y))
            .map(|c| c.set_symbol("┤").set_fg(border_color));
        buf.cell_mut((div_x, tabs_bottom_y))
            .map(|c| c.set_symbol("┤").set_fg(border_color));
        // Lado izquierdo (borde outer): ├
        buf.cell_mut((area.x, brand_bottom_y))
            .map(|c| c.set_symbol("├").set_fg(outer_color));
        buf.cell_mut((area.x, tabs_bottom_y))
            .map(|c| c.set_symbol("├").set_fg(outer_color));
    }

    if app.show_event_log {
        render_event_log(f, app, area);
    }
}

fn render_event_log(f: &mut Frame, app: &App, area: Rect) {
    let panel_w = (area.width / 3).max(30).min(60);
    let panel_h = (area.height * 2 / 3).max(8).min(30);
    let x = area.x + area.width.saturating_sub(panel_w + 2);
    let y = area.y + area.height.saturating_sub(panel_h + 2);
    let panel_area = Rect::new(x, y, panel_w, panel_h);

    let bc = app.brand_color();
    let block = Block::default()
        .title(Span::styled(
            format!(" events [{}]  [e] close ", app.event_log.len()),
            Style::default().fg(bc).add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(app.border_type())
        .border_style(Style::default().fg(bc));

    let inner = block.inner(panel_area);
    f.render_widget(Clear, panel_area);
    f.render_widget(block, panel_area);

    let items: Vec<ListItem> = app.event_log.iter().map(|e| {
        let kind_color = match e.kind {
            EventKind::Error   => C_RED,
            EventKind::Success => C_GREEN,
            EventKind::Info    => C_CYAN,
        };
        let kind_sym = match e.kind {
            EventKind::Error   => "✗",
            EventKind::Success => "✓",
            EventKind::Info    => "·",
        };
        ListItem::new(Line::from(vec![
            Span::styled(format!(" {} ", e.timestamp), Style::default().fg(C_DIM)),
            Span::styled(kind_sym, Style::default().fg(kind_color)),
            Span::raw(" "),
            Span::styled(&e.message, Style::default().fg(C_WHITE)),
        ]))
    }).collect();

    f.render_widget(List::new(items), inner);
}

fn render_hint(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let bc = app.brand_color();
    if app.sidebar_focused {
        let line = Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" open  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Esc]", Style::default().fg(bc)),
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
                    Span::styled("[space]", Style::default().fg(bc)),
                    Span::styled(" unstage  ", Style::default().fg(C_SUBTLE)),
                    Span::styled("[d]", Style::default().fg(bc)),
                    Span::styled(" diff", Style::default().fg(C_SUBTLE)),
                ]),
                Panel::Unstaged => Line::from(vec![
                    Span::raw(" "),
                    Span::styled("[space]", Style::default().fg(bc)),
                    Span::styled(" stage  ", Style::default().fg(C_SUBTLE)),
                    Span::styled("[d]", Style::default().fg(bc)),
                    Span::styled(" diff", Style::default().fg(C_SUBTLE)),
                ]),
                Panel::Untracked => Line::from(vec![
                    Span::raw(" "),
                    Span::styled("[space]", Style::default().fg(bc)),
                    Span::styled(" stage", Style::default().fg(C_SUBTLE)),
                ]),
                Panel::Log => Line::from(vec![
                    Span::raw(" "),
                    Span::styled("[d]", Style::default().fg(bc)),
                    Span::styled(" diff  ", Style::default().fg(C_SUBTLE)),
                    Span::styled("[l]", Style::default().fg(bc)),
                    Span::styled(" expand", Style::default().fg(C_SUBTLE)),
                ]),
            }
        }
        View::Commit => Line::from(vec![
            Span::raw(" "),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" save  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[←→]", Style::default().fg(bc)),
            Span::styled(" cursor  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Esc]", Style::default().fg(bc)),
            Span::styled(" cancel", Style::default().fg(C_SUBTLE)),
        ]),
        View::Sync => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" select  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" run  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Esc]", Style::default().fg(bc)),
            Span::styled(" cancel", Style::default().fg(C_SUBTLE)),
        ]),
        View::Log => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[d]", Style::default().fg(bc)),
            Span::styled(" diff", Style::default().fg(C_SUBTLE)),
        ]),
        View::Branch => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" checkout", Style::default().fg(C_SUBTLE)),
        ]),
        View::Snapshot => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" restore", Style::default().fg(C_SUBTLE)),
        ]),
        View::Tag => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" push  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[d]", Style::default().fg(bc)),
            Span::styled(" delete", Style::default().fg(C_SUBTLE)),
        ]),
        View::History => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" cherry-pick", Style::default().fg(C_SUBTLE)),
        ]),
        View::Remote => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" info", Style::default().fg(C_SUBTLE)),
        ]),
        View::Mirror => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" sync", Style::default().fg(C_SUBTLE)),
        ]),
        View::Workspace => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" sync all", Style::default().fg(C_SUBTLE)),
        ]),
        View::Config => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" edit  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Tab]", Style::default().fg(bc)),
            Span::styled(" toggle scope", Style::default().fg(C_SUBTLE)),
        ]),
        View::Settings => Line::from(vec![
            Span::raw(" "),
            Span::styled("[↑↓/jk]", Style::default().fg(bc)),
            Span::styled(" navigate  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[Enter]", Style::default().fg(bc)),
            Span::styled(" toggle/edit  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[s]", Style::default().fg(bc)),
            Span::styled(" save", Style::default().fg(C_SUBTLE)),
        ]),
        _ => Line::from(""),
    };
    f.render_widget(Paragraph::new(line), area);
}

fn render_sidebar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let border_color = if app.sidebar_focused { app.brand_color() } else { C_BORDER };

    // Right border as divider between sidebar and content
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
        Line::from(Span::styled("⛩  gitorii", Style::default().fg(app.brand_color()).add_modifier(Modifier::BOLD))),
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
        let sel_bg = app.selected_bg();
        let brand  = app.brand_color();

        let (prefix, label_style, bg) = if is_current_view {
            ("█ ", Style::default().fg(C_WHITE).add_modifier(Modifier::BOLD), Some(sel_bg))
        } else if is_sidebar_sel {
            ("▶ ", Style::default().fg(C_WHITE), Some(sel_bg))
        } else {
            ("  ", Style::default().fg(C_SUBTLE), None)
        };

        let mut item = ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(brand)),
            Span::styled(tab.label, label_style),
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
            Span::styled("help  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[?]", Style::default().fg(BRAND_COLOR)),
        ])),
        ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled("quit  ", Style::default().fg(C_SUBTLE)),
            Span::styled("[q]", Style::default().fg(BRAND_COLOR)),
        ])),
    ]);
    f.render_widget(
        bottom.block(Block::default().borders(Borders::NONE)),
        rows[2],
    );
}
