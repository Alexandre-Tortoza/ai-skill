# Dependency Rule

## The Rule

```
tui → core ← adapters
```

- `tui` depends on `core` and `adapters`
- `adapters` depends on `core`
- `core` depends on **nothing** outside the standard library

## Enforcement

Cargo enforces this at the crate level via `Cargo.toml`:

```toml
# tui/Cargo.toml
[dependencies]
ai-skill-core = { path = "../core" }
ai-skill-adapters = { path = "../adapters" }

# adapters/Cargo.toml
[dependencies]
ai-skill-core = { path = "../core" }

# core/Cargo.toml
# (no workspace-internal dependencies)
```

A `use ai_skill_adapters::...` in `core/src/lib.rs` would fail to compile.

## What Core Must Never Import

```
❌ std::fs          — filesystem access
❌ std::process     — command execution
❌ std::net         — network access
❌ ratatui          — rendering library
❌ crossterm        — terminal control
❌ notify           — file watcher
❌ ai_skill_adapters — adapter crate
❌ ai_skill   — UI crate
```

## Why This Matters

### Testability

Pure domain functions in `core` can be tested with zero I/O. The 78 unit tests in `core` complete in ~10ms with no temp files, fixtures, or environment setup.

### Compile-Time Contract

If a domain function needs data from the filesystem, it must go through a port trait:

```rust
// core — pure, testable
pub trait SkillRepository {
    fn list(&self) -> Result<Vec<Skill>, Self::Error>;
}

pub fn audit_skills(skills: &[Skill]) -> AuditReport<'_> {
    // pure logic, no I/O
}
```

The adapter provides the I/O:

```rust
// adapters — implements the port
impl SkillRepository for FsSkillRepository {
    fn list(&self) -> Result<Vec<Skill>, RepositoryError> {
        // filesystem access here
    }
}
```

### Swappable Implementations

The `App` struct in `tui` is generic over gateway, installer, and toggler:

```rust
pub struct App<G: AnyCatalogGateway, I: SkillInstaller, T: SkillToggler> { ... }
```

This allows:
- Real adapters in production
- Fake/stub adapters in tests
- Future adapters (e.g., HTTP-based repository, different installer)

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [Crates](crates.md) · [Decisions](decisions.md)
