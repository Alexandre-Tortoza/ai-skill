//! Terminal event polling and application-level event types.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
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

/// True for keys that always quit regardless of view (q, Ctrl-C).
/// Esc is intentionally excluded — it is handled contextually per view.
pub fn is_quit(key: &KeyEvent) -> bool {
    matches!(
        (key.code, key.modifiers),
        (KeyCode::Char('q'), KeyModifiers::NONE) | (KeyCode::Char('c'), KeyModifiers::CONTROL)
    )
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

    fn key_with(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn is_quit_lowercase_q() {
        assert!(is_quit(&key(KeyCode::Char('q'))));
    }

    #[test]
    fn is_quit_ctrl_c() {
        assert!(is_quit(&key_with(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL
        )));
    }

    #[test]
    fn is_quit_uppercase_q_is_not_quit() {
        assert!(!is_quit(&key_with(KeyCode::Char('Q'), KeyModifiers::SHIFT)));
    }

    #[test]
    fn is_quit_enter_is_not_quit() {
        assert!(!is_quit(&key(KeyCode::Enter)));
    }

    #[test]
    fn is_quit_q_with_shift_is_not_quit() {
        assert!(!is_quit(&key_with(KeyCode::Char('q'), KeyModifiers::SHIFT)));
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
