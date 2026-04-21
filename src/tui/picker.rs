use std::io;
use std::path::{Path, PathBuf};
use crossterm::{
    execute,
    event::{self, Event, KeyCode, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

const BRAND_COLOR: Color = Color::Rgb(255, 76, 76);
const SELECTED_BG: Color = Color::Rgb(40, 40, 60);
const C_WHITE: Color     = Color::Rgb(220, 220, 220);
const C_SUBTLE: Color    = Color::Rgb(140, 140, 160);
const C_DIM: Color       = Color::Rgb(80, 80, 100);
const C_GREEN: Color     = Color::Rgb(100, 220, 100);
const C_YELLOW: Color    = Color::Rgb(255, 210, 80);
const C_BORDER: Color    = Color::Rgb(60, 60, 80);

pub enum PickerResult {
    SingleRepo(PathBuf),
    Workspace { name: String, repos: Vec<PathBuf> },
    Cancelled,
}

enum PickerMode {
    Selecting,
    NamingWorkspace,
}

pub fn run_picker(start_dir: &Path) -> crate::error::Result<PickerResult> {
    let repos = find_git_repos(start_dir, 3);
    if repos.is_empty() {
        return Ok(PickerResult::Cancelled);
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut idx = 0usize;
    let mut selected: Vec<bool> = vec![false; repos.len()];
    let mut mode = PickerMode::Selecting;
    let mut ws_name = default_ws_name(start_dir);
    let mut ws_cursor = ws_name.len();

    let result = loop {
        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                ])
                .split(area);

            // Header
            f.render_widget(
                Paragraph::new(Line::from(vec![
                    Span::styled("⛩  gitorii", Style::default().fg(BRAND_COLOR).add_modifier(Modifier::BOLD)),
                    Span::styled("  — selecciona repositorios", Style::default().fg(C_SUBTLE)),
                ]))
                .block(Block::default().borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(BRAND_COLOR))),
                chunks[0],
            );

            // Repo list
            let n_sel = selected.iter().filter(|&&s| s).count();
            let items: Vec<ListItem> = repos.iter().enumerate().map(|(i, p)| {
                let is_cur = i == idx;
                let is_sel = selected[i];
                let style = if is_cur {
                    Style::default().bg(SELECTED_BG).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let check = if is_sel { "◆ " } else { "◇ " };
                let name = p.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| p.to_string_lossy().to_string());
                let path_str = p.to_string_lossy().to_string();
                ListItem::new(Line::from(vec![
                    Span::styled(if is_cur { "▶ " } else { "  " }, Style::default().fg(BRAND_COLOR)),
                    Span::styled(check, Style::default().fg(if is_sel { C_GREEN } else { C_DIM })),
                    Span::styled(format!("{:<24}", name), Style::default().fg(if is_cur { C_WHITE } else { C_SUBTLE })),
                    Span::styled(path_str, Style::default().fg(C_DIM)),
                ])).style(style)
            }).collect();

            let list_title = format!(" repositorios ({} encontrados, {} seleccionados) ", repos.len(), n_sel);
            f.render_widget(
                List::new(items).block(Block::default()
                    .title(Span::styled(list_title, Style::default().fg(C_SUBTLE)))
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(C_BORDER))),
                chunks[1],
            );

            // Workspace name input (always visible, editable when naming)
            let (ws_label, ws_style, ws_border) = match mode {
                PickerMode::NamingWorkspace => (
                    " nombre del workspace ",
                    Style::default().fg(BRAND_COLOR).add_modifier(Modifier::BOLD),
                    Style::default().fg(BRAND_COLOR),
                ),
                PickerMode::Selecting => (
                    " nombre del workspace (auto) ",
                    Style::default().fg(C_DIM),
                    Style::default().fg(C_BORDER),
                ),
            };

            let name_content = match mode {
                PickerMode::NamingWorkspace => {
                    let before = &ws_name[..ws_cursor.min(ws_name.len())];
                    let cur_char = ws_name[ws_cursor.min(ws_name.len())..].chars().next().unwrap_or(' ');
                    let after = if ws_cursor >= ws_name.len() { "" } else {
                        &ws_name[ws_cursor + cur_char.len_utf8()..]
                    };
                    Line::from(vec![
                        Span::raw(" "),
                        Span::styled(before, Style::default().fg(C_WHITE)),
                        Span::styled(cur_char.to_string(), Style::default().bg(BRAND_COLOR).fg(C_WHITE)),
                        Span::styled(after, Style::default().fg(C_WHITE)),
                    ])
                }
                PickerMode::Selecting => Line::from(vec![
                    Span::raw(" "),
                    Span::styled(&ws_name, Style::default().fg(C_DIM)),
                ]),
            };

            f.render_widget(
                Paragraph::new(name_content).block(Block::default()
                    .title(Span::styled(ws_label, ws_style))
                    .borders(Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(ws_border)),
                chunks[2],
            );

            // Hint
            let hint = match mode {
                PickerMode::Selecting => {
                    let n = selected.iter().filter(|&&s| s).count();
                    if n == 1 {
                        Line::from(vec![
                            Span::styled("[space]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" marcar  ", Style::default().fg(C_SUBTLE)),
                            Span::styled("[a]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" todos  ", Style::default().fg(C_SUBTLE)),
                            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" abrir repo  ", Style::default().fg(C_SUBTLE)),
                            Span::styled("[q]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" salir", Style::default().fg(C_SUBTLE)),
                        ])
                    } else if n > 1 {
                        Line::from(vec![
                            Span::styled("[space]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" marcar  ", Style::default().fg(C_SUBTLE)),
                            Span::styled("[a]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" todos  ", Style::default().fg(C_SUBTLE)),
                            Span::styled("[A]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" ninguno  ", Style::default().fg(C_SUBTLE)),
                            Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" crear workspace  ", Style::default().fg(C_SUBTLE)),
                            Span::styled("[q]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" salir", Style::default().fg(C_SUBTLE)),
                        ])
                    } else {
                        Line::from(vec![
                            Span::styled("[↑↓/jk]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" navegar  ", Style::default().fg(C_SUBTLE)),
                            Span::styled("[space]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" marcar  ", Style::default().fg(C_SUBTLE)),
                            Span::styled("[a]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" todos  ", Style::default().fg(C_SUBTLE)),
                            Span::styled("[q]", Style::default().fg(BRAND_COLOR)),
                            Span::styled(" salir", Style::default().fg(C_SUBTLE)),
                        ])
                    }
                }
                PickerMode::NamingWorkspace => Line::from(vec![
                    Span::styled("[Enter]", Style::default().fg(BRAND_COLOR)),
                    Span::styled(" confirmar  ", Style::default().fg(C_SUBTLE)),
                    Span::styled("[Esc]", Style::default().fg(BRAND_COLOR)),
                    Span::styled(" cancelar", Style::default().fg(C_SUBTLE)),
                ]),
            };
            f.render_widget(Paragraph::new(hint), chunks[3]);
        })?;

        if !event::poll(std::time::Duration::from_millis(200))? { continue; }
        if let Event::Key(key) = event::read()? {
            match mode {
                PickerMode::Selecting => match (key.modifiers, key.code) {
                    (_, KeyCode::Char('q')) |
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        break PickerResult::Cancelled;
                    }
                    (_, KeyCode::Up) | (_, KeyCode::Char('k')) => {
                        if idx > 0 { idx -= 1; }
                    }
                    (_, KeyCode::Down) | (_, KeyCode::Char('j')) => {
                        if idx + 1 < repos.len() { idx += 1; }
                    }
                    (_, KeyCode::Char(' ')) => {
                        selected[idx] = !selected[idx];
                    }
                    (_, KeyCode::Char('a')) => {
                        let all = selected.iter().all(|&s| s);
                        selected.iter_mut().for_each(|s| *s = !all);
                    }
                    (KeyModifiers::SHIFT, KeyCode::Char('A')) |
                    (_, KeyCode::Char('A')) => {
                        selected.iter_mut().for_each(|s| *s = false);
                    }
                    (_, KeyCode::Enter) => {
                        let sel_repos: Vec<PathBuf> = repos.iter().enumerate()
                            .filter(|(i, _)| selected[*i])
                            .map(|(_, p)| p.clone())
                            .collect();
                        if sel_repos.len() == 1 {
                            break PickerResult::SingleRepo(sel_repos.into_iter().next().unwrap());
                        } else if sel_repos.len() > 1 {
                            mode = PickerMode::NamingWorkspace;
                            ws_cursor = ws_name.len();
                        }
                        // 0 seleccionados: no hace nada
                    }
                    _ => {}
                },
                PickerMode::NamingWorkspace => match (key.modifiers, key.code) {
                    (_, KeyCode::Esc) => {
                        mode = PickerMode::Selecting;
                    }
                    (_, KeyCode::Enter) => {
                        let name = if ws_name.trim().is_empty() {
                            default_ws_name(start_dir)
                        } else {
                            ws_name.trim().to_string()
                        };
                        let sel_repos: Vec<PathBuf> = repos.iter().enumerate()
                            .filter(|(i, _)| selected[*i])
                            .map(|(_, p)| p.clone())
                            .collect();
                        break PickerResult::Workspace { name, repos: sel_repos };
                    }
                    (_, KeyCode::Backspace) => {
                        if ws_cursor > 0 {
                            ws_name.remove(ws_cursor - 1);
                            ws_cursor -= 1;
                        }
                    }
                    (_, KeyCode::Left) => {
                        if ws_cursor > 0 { ws_cursor -= 1; }
                    }
                    (_, KeyCode::Right) => {
                        if ws_cursor < ws_name.len() { ws_cursor += 1; }
                    }
                    (_, KeyCode::Char(c)) if key.modifiers == KeyModifiers::NONE
                                          || key.modifiers == KeyModifiers::SHIFT => {
                        ws_name.insert(ws_cursor, c);
                        ws_cursor += 1;
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        break PickerResult::Cancelled;
                    }
                    _ => {}
                },
            }
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(result)
}

fn find_git_repos(base: &Path, max_depth: usize) -> Vec<PathBuf> {
    let mut repos = Vec::new();
    scan_dir(base, base, 0, max_depth, &mut repos);
    repos.sort();
    repos
}

const SKIP_DIRS: &[&str] = &[
    "node_modules", "target", ".venv", "venv", "dist", "build", ".next",
    "__pycache__", ".cache", ".parcel-cache",
];

fn load_ignore_patterns(dir: &Path) -> Vec<String> {
    let names = [".toriignore", ".gitignore"];
    for name in &names {
        let p = dir.join(name);
        if let Ok(content) = std::fs::read_to_string(&p) {
            return content
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect();
        }
    }
    vec![]
}

fn is_ignored(name: &str, patterns: &[String]) -> bool {
    for pat in patterns {
        let pat = pat.trim_end_matches('/');
        if pat == name { return true; }
        if pat.starts_with('!') { continue; }
        // simple glob: *.ext
        if let Some(ext) = pat.strip_prefix("*.") {
            if name.ends_with(&format!(".{}", ext)) { return true; }
        }
    }
    false
}

fn has_git(dir: &Path) -> bool {
    // .git puede ser carpeta (repo normal) o archivo (submodule/worktree gitfile)
    dir.join(".git").exists()
}

fn scan_dir(base: &Path, dir: &Path, depth: usize, max_depth: usize, out: &mut Vec<PathBuf>) {
    if depth > max_depth { return; }
    if has_git(dir) {
        if dir != base {
            out.push(dir.to_path_buf());
        }
        return;
    }
    let ignore = load_ignore_patterns(dir);
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if name.starts_with('.') { continue; }
        if SKIP_DIRS.contains(&name.as_str()) { continue; }
        if is_ignored(&name, &ignore) { continue; }
        scan_dir(base, &path, depth + 1, max_depth, out);
    }
}

fn default_ws_name(dir: &Path) -> String {
    let base = dir.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_string());
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    format!("{}-{}", base, date)
}

pub fn save_workspace(name: &str, repos: &[PathBuf]) -> crate::error::Result<()> {
    let path = dirs::home_dir()
        .map(|h| h.join(".torii/workspaces.toml"))
        .unwrap_or_default();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut content = std::fs::read_to_string(&path).unwrap_or_default();
    content.push_str(&format!("\n[{}]\n", name));
    for repo in repos {
        content.push_str(&format!("path = \"{}\"\n", repo.display()));
    }
    std::fs::write(&path, content)?;
    Ok(())
}
