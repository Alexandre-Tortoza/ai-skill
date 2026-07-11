//! Terminal setup, teardown, and panic hook for the TUI.

use crossterm::{
    cursor,
    event::EnableMouseCapture,
    execute,
    terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};
use thiserror::Error;

/// Errors that can occur during terminal setup / teardown.
#[derive(Error, Debug)]
pub enum TerminalError {
    #[error("terminal io error: {0}")]
    Io(#[from] io::Error),
}

/// Convenience alias for the concrete terminal type.
pub type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Installs a panic hook that restores the terminal before the standard panic output.
///
/// Must be called before [`setup()`] so panics always restore the terminal.
pub fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(
            io::stderr(),
            crossterm::terminal::LeaveAlternateScreen,
            cursor::Show
        );
        disable_raw_mode().ok();
        original(info);
    }));
}

/// Enters raw mode and the alternate screen, returning a [`Terminal`].
pub fn setup() -> Result<AppTerminal, TerminalError> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

/// Leaves alternate screen, disables raw mode, and shows the cursor again.
pub fn teardown(terminal: &mut AppTerminal) -> Result<(), TerminalError> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture,
        cursor::Show,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn terminal_error_io_construction() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "test");
        let err = TerminalError::from(io_err);
        assert!(err.to_string().contains("terminal io error"));
    }

    #[test]
    fn terminal_error_io_debug() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "test");
        let err = TerminalError::Io(io_err);
        assert!(!format!("{err:?}").is_empty());
    }

    #[test]
    fn install_panic_hook_does_not_panic() {
        install_panic_hook();
        let hook = std::panic::take_hook();
        std::panic::set_hook(hook);
    }
}
