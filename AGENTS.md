# ai-skill

Early-stage Rust TUI for managing AI agent skills (Claude Code, Cursor, etc.).

## Stack & architecture

- **Rust edition 2024**, workspace Cargo com crates `core`, `adapters` e `tui`
- Dependencies in use include `ratatui`, `crossterm`, `serde`, `notify`, `thiserror`, `serde_norway`, `insta` and `tempfile`
- **Hexagonal architecture**: `core` (pure domain) / `adapters` (FS, catalog, installer) / `tui`
- Dependency flow: `tui` -> `core` <- `adapters`

## Current state

The functional backlog waves 0-4 are implemented locally. Current focus is repository health: docs, CI, release surface, packaging, automation and cross-cutting polish.

## Commands

```sh
cargo build
cargo run -p ai-skill
cargo test
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
```

## Methodology

XP + TDD (strict Red → Green → Refactor). Vertical slices, each story labeled `[U]`/`[I]`/`[E]` for unit/integration/e2e.

## References

- `docs/installation.md` — installation guide
- `docs/usage.md` — TUI user guide
- `docs/architecture.md` — crate boundaries, ports/adapters and shell-out decision
- `docs/development.md` — developer setup and methodology
- `docs/security.md` — security model and heuristic scan
- `docs/api.md` — public API surface per crate
- `docs/roadmap.md` — backlog with 5 waves (Ondas 0–4) + health checklist
- `docs/features.md` — icebox of future features, ordered by priority groups A–G
- `docs/review-prompts.md` — LLM review prompts for PRs
- The roadmap's health checklist enumerates CI/release/DX expectations not yet implemented

## Conventions

- Model domain types for every port (no leaky primitives)
- Error handling: fail-fast with actionable messages
- TUI respects `NO_COLOR`, works at 80×24
- Prefer `shell out` for external tooling (e.g. `npx skills` for remote catalog)
- License is AGPL-3.0-only
