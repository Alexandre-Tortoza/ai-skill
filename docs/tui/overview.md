# TUI Overview

`ai-skill` is a binary crate that provides the terminal interface. It wires concrete adapters into the domain use cases and renders a `ratatui` interface.

## Package

```toml
[dependencies]
ai-skill-core = { path = "../core" }
ai-skill-adapters = { path = "../adapters" }
ratatui = "0.29"
crossterm = "0.28"
thiserror = "2"

[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
```

## Module Map

| Module | Files | Purpose |
|---|---|---|
| `main.rs` | (binary root) | Entry point, adapter wiring, event loop |
| `app` | `app.rs` | `App<G,I,T>` state machine, 1971 lines |
| `event` | `event.rs` | `AppEvent`, keyboard polling |
| `terminal` | `terminal.rs` | Setup/teardown, panic hook |
| `ui` | `ui/mod.rs` + 14 panel files | Rendering functions |

## Entry Point Flow

```
main()
├── Initialize adapters (FsSkillRepository, GitDriftChecker, FsWatcher, ...)
├── Create App with injected adapters
├── terminal::setup() → alternate screen, raw mode
├── Main loop:
│   ├── Poll watcher for filesystem changes
│   ├── Poll crossterm for keyboard events
│   ├── app.handle_event(event)
│   ├── Render active view:
│   │   ├── Panel render function for current view
│   │   └── Status bar at bottom
│   └── Loop until app.should_quit
└── terminal::teardown() → restore terminal
```

## View Dispatch

The `main.rs` matches `app.view` and calls the corresponding panel renderer, then always renders the status bar:

```rust
match app.view {
    View::List         => installed_panel::render_installed_panel(...),
    View::Detail       => detail_panel::render_detail_panel(...),
    View::Search       => search_panel::render_search_panel(...),
    View::Help         => { installed_panel::render_installed_panel(...);
                            help_overlay::render_help_overlay(...); }
    View::Confirm      => { installed_panel::render_installed_panel(...);
                            confirm_panel::render_confirm_panel(...); }
    View::InstallWizard => install_wizard::render_install_wizard(...),
    View::ScanReport   => scan_report::render_scan_report(...),
    View::Profiles     => profiles_panel::render_profiles_panel(...),
    View::CreateWizard => create_wizard::render_create_wizard(...),
    View::Editor       => editor_panel::render_editor_panel(...),
    View::Audit        => audit_panel::render_audit_panel(...),
}
render_status_bar(&app.view, ...);
```

Help and Confirm views overlay on top of the List panel (the list stays visible beneath the popup).

---

[← Back to index](../index.md) · Related: [App State](app-state.md) · [Views](views.md) · [Terminal](terminal.md)
