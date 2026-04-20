use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::time::Duration;
use crate::error::Result;
use super::app::App;

pub enum Action {
    Quit,
    Refresh,
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn next(&mut self, app: &mut App) -> Result<Option<Action>> {
        if !event::poll(Duration::from_millis(200))? {
            return Ok(None);
        }

        match event::read()? {
            Event::Key(key) => {
                match (key.modifiers, key.code) {
                    // Quit
                    (_, KeyCode::Char('q')) |
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        return Ok(Some(Action::Quit));
                    }

                    // Navigation between panels
                    (_, KeyCode::Tab) => app.next_panel(),
                    (KeyModifiers::SHIFT, KeyCode::BackTab) => app.prev_panel(),

                    // Navigation within panel
                    (_, KeyCode::Up)   | (_, KeyCode::Char('k')) => app.move_up(),
                    (_, KeyCode::Down) | (_, KeyCode::Char('j')) => app.move_down(),

                    // Refresh
                    (_, KeyCode::Char('r')) => return Ok(Some(Action::Refresh)),

                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }

        Ok(None)
    }
}
