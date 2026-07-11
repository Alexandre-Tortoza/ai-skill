# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`ai-skill` is a terminal UI for managing Claude Code skills and those of other AI agents. Stack: Rust + `ratatui` + `crossterm` + `tokio` + `serde` + `walkdir` + `notify`. Remote catalog data is fetched by shelling out to `npx skills`; trending/auth-gated routes are out of scope for now.

## Commands

```bash
cargo build          # compile
cargo test           # run all tests
cargo test <name>    # run a single test by name (substring match)
cargo fmt --check    # check formatting
cargo clippy -D warnings  # lint (warnings are errors)
cargo audit          # check for known vulnerabilities
```

The project is structured as a Cargo workspace with three crates: `core`, `adapters`, `tui`.

## Architecture

Hexagonal (ports-and-adapters):

- **`core`** — pure domain, zero I/O. Contains `Skill`, `ValidationState`, `Profile`, `Installer` port, security-scan logic, drift detection, audit report. All business rules live here; test with unit tests only (`[U]`).
- **`adapters`** — implements the core ports against real I/O: `FsSkillRepository` (walks `~/.claude/skills` and `.claude/skills`, resolves symlinks), `CatalogGateway` (shells out to `npx skills find`), `CliInstaller` (shells out to `npx skills add/remove/update`), `notify`-based file watcher. Test with integration tests against fixture directories (`[I]`).
- **`tui`** — `ratatui`/`crossterm` UI. Panels: Instaladas (installed skills), Buscar (search), Profiles/Presets, detail pane, editor split, creation wizard. Test with render snapshots (`[E]`).

Dependency flow: `tui` → `core` ← `adapters`. `core` never imports `adapters` or `tui`.

## Engineering principles

### TDD / XP discipline
Strict **Red → Green → Refactor** on every story. Write the failing test first; make it pass with the simplest code that works; only then clean up. Do not skip Refactor — it is where design happens. Each story is a thin vertical slice deliverable end-to-end. Pair when complexity warrants it; prefer collective ownership (no "my code" areas).

Test labels in the backlog: `[U]` = unit test in `core`, `[I]` = integration test in `adapters`, `[E]` = TUI render/snapshot test.

### SOLID (applied to this codebase)
- **S** — each type has one reason to change. `Skill` models data; `FsSkillRepository` reads it; `ValidationService` classifies it. Never merge these.
- **O** — extend by adding new adapter implementations or new port variants, not by editing existing ones. New agent support = new adapter, not a new `if` branch in existing code.
- **L** — any `SkillRepository` implementation must satisfy the same invariants (same error types, same ordering guarantees). Do not add hidden postconditions in concrete types.
- **I** — define narrow ports (`SkillReader`, `SkillWriter`, `Installer`) instead of fat interfaces; consumers only depend on what they use.
- **D** — `core` depends on traits (`SkillRepository`, `Installer`); `adapters` provides the concrete types. Never import a concrete adapter type inside `core`.

### Clean Architecture / Hexagonal
The dependency rule is absolute: **nothing in `core` may import from `adapters` or `tui`**. I/O lives exclusively in adapters. The `core` compiles and passes all its tests without touching the filesystem or network.

Ports are traits defined in `core`. Adapters implement them. The TUI drives the use-case by calling `core` functions with injected adapters.

### KISS
Prefer the simplest structure that makes the test pass. A plain `Vec` beats a `HashMap` until profiling says otherwise. A synchronous function beats an `async` one unless the call actually blocks. Remove indirection that does not earn its keep.

### DRY
Every piece of knowledge has one authoritative home. If the same validation rule appears in two places, extract it to `core` and call it from both. Do not duplicate fixture-building logic across tests — use builder helpers.

### YAGNI
Do not implement icebox items (see `docs/features.md`) ahead of their wave. Do not add configuration knobs, extension points, or abstractions for hypothetical future consumers. Build exactly what the current story requires; the Refactor step will reveal the right abstraction if one is needed.

### Fail fast, explicit errors
Return `Result<T, E>` with a domain error enum at every boundary; never silently swallow errors or substitute defaults. Error variants must be actionable — they should tell the user what to do, not just what went wrong. `unwrap()` is only acceptable in tests.

### Immutability first
Prefer immutable data in `core`. Domain types (`Skill`, `Profile`) are value objects — construct them fully formed or return an error. Mutation lives in adapters (filesystem writes, watcher state).

### Continuous integration mindset
Every commit must leave `cargo fmt --check && cargo clippy -D warnings && cargo test && cargo audit` green. Never push red. Fix broken windows immediately — a warning left in is a warning normalized.

## Delivery waves

Stories are grouped in waves; do not pull from a later wave until all stories in the current wave are done:

- **Onda 0** — walking skeleton: workspace compiles, TUI opens, read-only inventory renders.
- **Onda 1** — MVP: validation states, duplicate detection, search panel, status bar.
- **Onda 2** — lifecycle management: install/remove/update, enable/disable, bulk actions, tags.
- **Onda 3** — differentiators: profiles, security scan, scan gate before install.
- **Onda 4** — continuous maintenance: drift detection, reactive watch, editor, audit report.

Future icebox items live in `docs/features.md`; do not implement them ahead of schedule.

## Repository health

Repository health work is tracked in `docs/roadmap.md`. Architecture decisions live in `docs/architecture.md`; LLM review prompts live in `docs/review-prompts.md`. The project license is AGPL-3.0-only.
