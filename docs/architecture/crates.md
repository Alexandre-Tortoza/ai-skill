# Crate Boundaries

The workspace has three crates: `core`, `adapters`, and `tui`. Each enforces a strict responsibility boundary.

## `ai-skill-core` — Pure Domain

```
core/Cargo.toml
  dependencies: serde, serde_norway, thiserror
  dev-dependencies: indoc
```

**What belongs here:**
- Domain models: `Skill`, `Scope`, `ValidationState`, `DriftState`, `Profile`, `CatalogEntry`
- Value objects: `SkillMetadata`, `ScanFinding`, `AuditReport`
- Port traits: `SkillRepository`, `AnyCatalogGateway`, `SkillInstaller`, `SkillToggler`, `ProfileStore`, `DriftChecker`, `SkillCreator`, `SkillWriter`
- Pure functions: `parse_frontmatter`, `scan_skill`, `detect_duplicates`, `diff_profile`, `audit_skills`, `scaffold_skill`, `apply_edit`

**What must NEVER appear:**
- Filesystem I/O (`std::fs`, `std::path::Path::read`...)
- External process execution (`std::process::Command`)
- Network access (`reqwest`, HTTP calls)
- Terminal libraries (`ratatui`, `crossterm`)
- File watching (`notify`)
- `unwrap()` on external data (unwrap on local proof is acceptable)

## `ai-skill-adapters` — I/O Implementations

```
adapters/Cargo.toml
  dependencies: ai-skill-core (path), notify, serde_norway, thiserror
  dev-dependencies: tempfile
```

**What belongs here:**
- Concrete implementations of core port traits
- Filesystem operations: directory scanning, symlink resolution, file reading/writing
- Process execution: shell out to `npx skills`
- Git operations: `git rev-parse HEAD`, `git rev-parse @{u}`
- File watching: debounced `notify` watcher

**Testing strategy:**
- Use `tempfile::TempDir` for isolated filesystem tests
- Git tests operate on real temporary git repos
- NPX-dependent tests are `#[ignore = "requires npx"]`

## `ai-skill` — Terminal Interface

```
tui/Cargo.toml
  dependencies: ai-skill-core (path), ai-skill-adapters (path),
                ratatui, crossterm, thiserror
  dev-dependencies: insta (yaml)
```

**What belongs here:**
- Binary entry point (`main.rs`) that wires adapters and starts the event loop
- Application state machine (`App<G, I, T>`) with view dispatching
- UI rendering panels (ratatui `Widget` functions)
- Terminal lifecycle management (alternate screen, raw mode, panic recovery)
- Event polling (keyboard input, resize events, watcher signals)

**What does NOT belong:**
- Domain logic (delegate to `core`)
- Direct I/O outside of adapter instances (use injected adapters)

**Testing strategy:**
- Render snapshots with `insta` using ratatui's `TestBackend`
- State machine tests with fake adapters (strings as stubs)

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [Dependency Rule](dependency-rule.md)
