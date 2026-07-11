# Core Crate Overview

`ai-skill-core` is the pure domain layer. It has zero I/O dependencies — no filesystem, no network, no terminal, no external processes.

## Module Map

| Module | Items | Purpose |
|---|---|---|
| `skill` | `Skill`, `Scope` | Core domain models |
| `validation` | `ValidationState` | Skill health states |
| `frontmatter` | `parse_frontmatter`, `extract_body`, `SkillMetadata`, `ParseError` | YAML frontmatter parsing |
| `repository` | `SkillRepository` | Port for listing skills |
| `catalog` | `AnyCatalogGateway`, `CatalogEntry` | Port for remote search |
| `installer` | `SkillInstaller`, `SkillToggler` | Ports for lifecycle mgmt |
| `profile` | `Profile`, `ProfileOp`, `ProfileStore`, `diff_profile` | Profile system |
| `creator` | `SkillCreator`, `SkillWriter`, `scaffold_skill`, `apply_edit` | Skill creation |
| `drift` | `DriftChecker`, `DriftState` | Drift detection |
| `duplicate_detector` | `detect_duplicates` | Name collision detection |
| `security_scan` | `scan_skill`, `ScanFinding`, `Severity`, `ScanCategory` | Heuristic scanner |
| `audit` | `audit_skills`, `AuditReport` | Aggregated audit |

## Package

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_norway = "0.9"
thiserror = "2"

[dev-dependencies]
indoc = "2"
```

No I/O libraries. No async. The crate is fully synchronous and deterministic.

## Design Principles

- **Ports over primitives**: every external dependency is modeled as a trait. Domain functions accept trait references, never concrete I/O types.
- **Fail-fast with types**: validation states, drift states, and parse errors are modeled as enums, not magic strings.
- **No leaky abstractions**: domain structs contain only domain fields. `PathBuf` is acceptable for paths; `String` for names. No ratatui `Rect`, no crossterm `KeyEvent` in domain types.
- **Deterministic**: same input → same output. No randomness, no time, no environment-dependent behavior.

---

[← Back to index](../index.md) · Related: [Skill Model](skill-model.md) · [Ports](ports.md)
