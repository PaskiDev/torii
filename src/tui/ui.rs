use ratatui::{Frame, style::Color};

use super::app::{App, View};
use super::views;

pub const BRAND_COLOR: Color = Color::Rgb(255, 76, 76);
pub const SELECTED_BG: Color = Color::Rgb(40, 40, 60);

pub fn render(f: &mut Frame, app: &App) {
    match app.view {
        View::Dashboard => views::dashboard::render(f, app),
        View::Diff      => views::diff::render(f, app),
        View::Log       => views::log::render(f, app),
        View::Branch    => views::branch::render(f, app),
        View::Commit    => views::commit::render(f, app),
        View::Snapshot  => views::snapshot::render(f, app),
        View::Sync      => views::sync::render(f, app),
    }
}
