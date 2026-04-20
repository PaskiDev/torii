pub mod app;
pub mod events;
pub mod ui;
pub mod views;

use std::io;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use events::EventHandler;

pub fn run() -> crate::error::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let mut events = EventHandler::new();

    let result = run_loop(&mut terminal, &mut app, &mut events);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    events: &mut EventHandler,
) -> crate::error::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        if let Some(action) = events.next(app)? {
            use events::Action;
            match action {
                Action::Quit => break,
                Action::Refresh => app.refresh()?,
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
