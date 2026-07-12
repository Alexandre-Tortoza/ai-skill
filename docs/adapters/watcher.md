# Watcher (`FsWatcher`)

Debounced filesystem watcher using the `notify` crate. Informs the TUI when skill files change on disk so it can refresh the view.

## Structure

```rust
pub struct FsWatcher {
    _watcher: RecommendedWatcher,   // notify watcher (kept alive)
    pub rx: Receiver<()>,           // public channel for events
    watched_paths: usize,           // existing roots actively watched
}
```

## Constructor

```rust
pub fn new(paths: &[PathBuf]) -> Result<Self, Box<dyn std::error::Error>>
```

- Creates a `notify::RecommendedWatcher` with debounced event mode
- Watches all provided paths (only those that exist)
- Counts how many roots were actually attached via `watched_paths()`
- Spawns a background thread to drain raw `notify::Event` stream
- Sends a `()` signal through a channel on every file change
- Debounce is handled by `notify` (configurable in `new` — currently 300ms)

## How the TUI Uses It

In `main.rs`, the event loop polls both `crossterm::event::poll` and the watcher channel:

```rust
loop {
    if watcher.rx.try_recv().is_ok() {
        // A file changed — refresh skill list
        app.all_skills = repository.list()?;
        app.needs_refresh = true;
    }
    // Poll keyboard events with 250ms timeout
    if let Some(event) = next_event(Duration::from_millis(250))? {
        app.handle_event(event);
    }
}
```

## Path Filtering

Only paths that exist at construction time are watched. If a root directory is created later, the watcher does not pick it up automatically (restart required).

## Hot-Reload Awareness

Claude Code 2.1+ reloads changed skills without restarting the agent. `ai-skill` reflects this live behavior by showing `reload:on` in the status bar when at least one existing skill root is actively watched.

## Thread Safety

The `notify` watcher runs on its own thread. Cross-thread communication uses `std::sync::mpsc`. The receiver is `pub rx: Receiver<()>` — the TUI owns it and polls it synchronously in the main loop.

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [FS Repository](fs-repository.md)
