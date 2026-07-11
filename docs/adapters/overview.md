# Adapters Overview

`ai-skill-adapters` implements the port traits defined in `core` using real I/O. Every adapter maps to exactly one port.

## Package

```toml
[dependencies]
ai-skill-core = { path = "../core" }
notify = "6"
serde_norway = "0.9"
thiserror = "2"

[dev-dependencies]
tempfile = "3"
```

## Adapter Inventory

| Adapter | Port | I/O Type | Constructor Pattern |
|---|---|---|---|
| `FsSkillRepository` | `SkillRepository` | Filesystem scan | `from_env()` / `new(global, project)` |
| `NpxCatalogGateway` | `AnyCatalogGateway` | Shell out | Unit struct |
| `CliInstaller` | `SkillInstaller` | Shell out | Unit struct |
| `FsToggler` | `SkillToggler` | Filesystem rename | Unit struct |
| `FsProfileStore` | `ProfileStore` | Filesystem YAML | `from_env()` / `new(base_dir)` |
| `GitDriftChecker` | `DriftChecker` | Git commands | Unit struct |
| `FsSkillCreator` | `SkillCreator` | Filesystem mkdir+write | `from_env()` / `new(base_dir)` |
| `FsSkillWriter` | `SkillWriter` | Filesystem write | Unit struct |

## Constructor Convention

Every adapter with configurable paths provides two constructors:

- **`new(...)`** — takes explicit paths for testability
- **`from_env()`** — resolves default paths from environment / home directory

This allows tests to inject temp directories while production uses standard paths.

## Error Types

```rust
pub enum RepositoryError {
    Io(#[from] std::io::Error),
}

pub enum CliInstallerError {
    Io(#[from] std::io::Error),
    NonZeroExit(i32),
}
```

Both implement `std::error::Error` via `thiserror` derive. Other adapters use `Box<dyn std::error::Error>` for simplicity.

---

[← Back to index](../index.md) · Related: [FS Repository](fs-repository.md) · [FS Skill Creator](fs-skill-creator.md) · [NPX Catalog](npx-catalog.md) · [CLI Installer](cli-installer.md)
