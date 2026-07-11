# Architecture Decisions

## Shell Out to `npx skills`

**Decision:** Use `std::process::Command` to shell out to `npx skills` for remote catalog operations instead of implementing a native HTTP client.

**Context:** The remote skill registry (skills.sh) has an evolving API with OIDC auth requirements and no stable public contract.

**Consequences:**
- Reduced coupling with unstable remote APIs
- Reuses authentication, caching, and behavior from upstream tooling
- Keeps MVP focused on inventory, audit, and local UX
- Process errors must become actionable user messages
- Tests must isolate external calls with fake adapters or controlled commands
- Features requiring OIDC auth (trending, community reputation) are out of scope

## Error Handling: Fail-Fast with Actionable Messages

**Decision:** Every boundary function returns `Result`. Error messages should indicate a concrete next step.

**Examples:**
- `"install missing dependency: npx not found in PATH"` ✓
- `"fix path: ~/.claude/skills does not exist"` ✓
- `"remove broken symlink at /path/to/skill"` ✓
- `"parse error"` ✗ (not actionable)

`unwrap()` is acceptable only in tests. In production, it requires local proof of impossibility where adding error handling would reduce clarity.

## Terminal at 80×24 Minimum

**Decision:** The TUI must remain usable at 80×24 columns. Below that, a resize message is shown instead of a broken layout.

**Rationale:** Terminal UIs are often used in SSH sessions, tmux splits, or small windows. 80×24 is the POSIX standard minimum.

## `NO_COLOR` and 16-Color Mode

**Decision:** Respect `NO_COLOR` environment variable. True color may enhance but must never be required to understand the UI.

## No Async Runtime in Core

**Decision:** `core` does not use `tokio`, `async-std`, or any async runtime. Synchronous pure functions keep the domain simple and testable.

**Exception:** `adapters` and `tui` use synchronous I/O with blocking calls. The event loop in `tui` uses a 250ms polling interval via `crossterm::event::poll`. If future performance requires async I/O, it stays in the adapter layer — `core` ports remain synchronous.

## Profile Diff Algorithm

**Decision:** Profile activation computes the diff between current installed skills and the desired profile, producing a minimal batch of install/remove operations.

**Details:**
- Only `ValidationState::Valid` skills count as "installed"
- Non-valid skills are ignored for removal (they're already broken)
- This prevents profiles from cascading errors from broken state

## Ignored Tests for External Dependencies

**Decision:** Tests requiring `npx`, a git remote, or specific user directories are marked `#[ignore = "..."]`.

**Rationale:** CI does not guarantee these prerequisites. The ignored tests document the contract for manual verification and local development.

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [Dependency Rule](dependency-rule.md)
