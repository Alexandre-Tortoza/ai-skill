//! Terminal event polling and application-level event types.

use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;
use thiserror::Error;

/// Wraps an I/O error from the terminal event system.
#[derive(Error, Debug)]
#[error("event error: {0}")]
pub struct EventError(#[from] std::io::Error);

/// Events that the application understands.
#[derive(Debug)]
pub enum AppEvent {
    /// A key press.
    Key(KeyEvent),
    /// Terminal resize.
    Resize,
}

/// Blocks for up to `timeout`, returns `None` if no event, or the next [`AppEvent`].
pub fn next_event(timeout: Duration) -> Result<Option<AppEvent>, EventError> {
    if !event::poll(timeout)? {
        return Ok(None);
    }
    match event::read()? {
        Event::Key(key) => Ok(Some(AppEvent::Key(key))),
        Event::Resize(_, _) => Ok(Some(AppEvent::Resize)),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn event_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err = EventError::from(io_err);
        assert!(err.to_string().contains("event error"));
    }

    #[test]
    fn event_error_debug() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "test");
        let err = EventError(io_err);
        assert!(!format!("{err:?}").is_empty());
    }

    #[test]
    fn app_event_resize_debug() {
        let event = AppEvent::Resize;
        assert!(!format!("{event:?}").is_empty());
    }

    #[test]
    fn app_event_key_debug() {
        let event = AppEvent::Key(key(KeyCode::Enter));
        assert!(!format!("{event:?}").is_empty());
    }
}
