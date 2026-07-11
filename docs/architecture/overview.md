# Architecture Overview

`ai-skill` uses **hexagonal architecture** (ports & adapters) to keep domain rules testable without filesystem, terminal, or network access.

## Rationale

AI skill management involves I/O-heavy operations (scanning directories, running shell commands, watching files) intertwined with pure domain logic (validation, duplicate detection, security scanning). Mixing them creates untestable code and hidden coupling.

Hexagonal architecture enforces a boundary:

- **Domain logic** lives in `core`, depends on nothing external.
- **I/O details** live in `adapters`, implement domain-defined interfaces.
- **Orchestration** lives in `tui`, wires adapters into use cases.

## Diagram

```
┌─────────────────────────────────────────────────┐
│                    tui                           │
│  (ratatui, crossterm, orchestration)            │
│         │                               │       │
│         │ calls core use cases          │ injects│
│         ▼                               ▼       │
│  ┌──────────┐                  ┌──────────────┐│
│  │  core    │◄──────────────── │  adapters    ││
│  │  (pure)  │  ports (traits)  │  (I/O impl)  ││
│  └──────────┘                  └──────────────┘│
└─────────────────────────────────────────────────┘
```

The arrow directions mean:
- `tui` depends on `core` (uses domain types and functions)
- `adapters` depends on `core` (implements port traits)
- `core` depends on **nothing** outside the standard library

## Ports and Adapters

| Port (core trait) | Adapter (concrete impl) | Purpose |
|---|---|---|
| `SkillRepository` | `FsSkillRepository` | List installed skills |
| `AnyCatalogGateway` | `NpxCatalogGateway` | Search remote catalog |
| `SkillInstaller` | `CliInstaller` | Install/remove/update |
| `SkillToggler` | `FsToggler` | Enable/disable/adopt |
| `ProfileStore` | `FsProfileStore` | Persist profiles |
| `DriftChecker` | `GitDriftChecker` | Detect upstream drift |
| `SkillCreator` | `FsSkillCreator` | Scaffold new skills |
| `SkillWriter` | `FsSkillWriter` | Write SKILL.md |

Every port is a Rust trait. Adapters are structs that implement those traits. The `tui` crate receives trait implementations via dependency injection (generic parameters or `Box<dyn Trait>`).

## Benefits

- **Testability**: `core` tests run in microseconds with no I/O. `adapters` tests use temp directories. `tui` tests use render snapshots.
- **Swapability**: adapters can be replaced without touching domain logic (e.g., `GitDriftChecker` → `HttpDriftChecker`).
- **Compile-time safety**: `core` cannot accidentally import `adapters` or `tui` — Cargo enforces this via crate dependency direction.

---

[← Back to index](../index.md) · Related: [Crates](crates.md) · [Dependency Rule](dependency-rule.md) · [Decisions](decisions.md)
