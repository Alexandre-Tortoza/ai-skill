# Terminal Lifecycle

The `terminal` module manages the crossterm terminal lifecycle: entering alternate screen, enabling raw mode, and restoring the terminal on exit or panic.

## Public API

```rust
pub type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

pub fn setup() -> Result<AppTerminal, TerminalError>
pub fn teardown(terminal: &mut AppTerminal) -> Result<(), TerminalError>
pub fn install_panic_hook()
```

## Setup

```rust
pub fn setup() -> Result<AppTerminal, TerminalError> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}
```

What happens:
1. **Raw mode**: keyboard input is processed immediately (no line buffering)
2. **Alternate screen**: the TUI is rendered on a separate screen buffer — when the app exits, the user's previous terminal content is restored
3. **Mouse capture**: mouse events are enabled (used for: future scroll support)

## Teardown

```rust
pub fn teardown(terminal: &mut AppTerminal) -> Result<(), TerminalError> {
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
```

Reverses setup:
1. Leave alternate screen → user sees their previous terminal content
2. Disable mouse capture
3. Disable raw mode → line buffering restored

## Panic Hook

```rust
pub fn install_panic_hook() {
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        // 1. Leave alternate screen
        // 2. Show cursor
        // 3. Disable raw mode
        // 4. Call previous panic hook (for backtrace)
        prev(panic);
    }));
}
```

If the application panics while the alternate screen is active, the user would be left in raw mode with no visible terminal. The panic hook ensures graceful restoration:

1. Write `LeaveAlternateScreen` and `Show` (cursor) escape codes directly to stderr
2. Disable raw mode via `crossterm::terminal::disable_raw_mode()`
3. Call the previous panic hook to print the panic message and backtrace

## Usage in `main.rs`

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    install_panic_hook();
    let mut terminal = setup()?;

    // ... event loop ...

    teardown(&mut terminal)?;
    Ok(())
}
```

## Errors

```rust
pub enum TerminalError {
    Io(#[from] io::Error),
}
```

All errors are I/O errors from crossterm operations.

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [App State](app-state.md)
