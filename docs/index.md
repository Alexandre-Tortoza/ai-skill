# ai-skill Documentation

Terminal UI for managing AI agent skills (Claude Code, Cursor, etc.).

## Architecture

| Note | Description |
|---|---|
| [Overview](architecture/overview.md) | Hexagonal architecture, ports & adapters pattern |
| [Crates](architecture/crates.md) | Crate boundaries and responsibilities |
| [Dependency Rule](architecture/dependency-rule.md) | Dependency flow and compile-time enforcement |
| [Decisions](architecture/decisions.md) | Architecture Decision Records (shell out, error handling, etc.) |

## Core Domain (`ai-skill-core`)

| Note | Description |
|---|---|
| [Overview](core/overview.md) | Pure domain crate — no I/O |
| [Skill Model](core/skill-model.md) | `Skill`, `Scope`, `ValidationState`, `DriftState` |
| [Ports](core/ports.md) | All 8 port traits with signatures |
| [Frontmatter](core/frontmatter.md) | YAML frontmatter parsing (`SKILL.md`) |
| [Security Scan](core/security-scan.md) | Heuristic scanner — patterns, categories, severities |
| [Profiles](core/profiles.md) | `Profile`, `ProfileOp`, diff algorithm |
| [Duplicate Detection](core/duplicate-detector.md) | Case-insensitive name collision detection |
| [Drift Detection](core/drift.md) | Git hash-based drift checking |
| [Audit](core/audit.md) | `AuditReport` — broken, duplicates, no-agents, updates |

## Adapters (`ai-skill-adapters`)

| Note | Description |
|---|---|
| [Overview](adapters/overview.md) | I/O implementations of core ports |
| [FS Repository](adapters/fs-repository.md) | `FsSkillRepository` — scanning and validation |
| [NPX Catalog](adapters/npx-catalog.md) | `NpxCatalogGateway` — remote search via `npx skills find` |
| [CLI Installer](adapters/cli-installer.md) | `CliInstaller` — install/remove/update via `npx skills` |
| [Toggler](adapters/toggler.md) | `FsToggler` — enable, disable, adopt |
| [Watcher](adapters/watcher.md) | `FsWatcher` — debounced filesystem watching |
| [FS Skill Creator](adapters/fs-skill-creator.md) | `FsSkillCreator`, `FsSkillWriter` — scaffold and write |
| [Git Drift](adapters/git-drift.md) | `GitDriftChecker` — `git rev-parse` based drift |
| [Profile Store](adapters/profile-store.md) | `FsProfileStore` — YAML profile persistence |

## TUI (`ai-skill`)

| Note | Description |
|---|---|
| [Overview](tui/overview.md) | Terminal interface crate |
| [App State](tui/app-state.md) | `App<G,I,T>`, `View` enum, state machine |
| [Views](tui/views.md) | All 11 panels — rendering and key bindings |
| [Terminal](tui/terminal.md) | Setup/teardown, panic hook, crossterm lifecycle |

## Development

| Note | Description |
|---|---|
| [Testing](dev/testing.md) | Strategy: Unit (U), Integration (I), E2E (E) |
| [Release](dev/release.md) | Release process, CI/CD pipelines, crates.io publishing |

## User Guides

| Note | Description |
|---|---|
| [Installation](installation.md) | Pre-built binaries, build from source, platform notes |
| [Usage](usage.md) | TUI user guide — views, key bindings, workflows |
| [Security](security.md) | Security model, heuristic scan, disclosure |

## Reference

| Note | Description |
|---|---|
| [API Reference](api.md) | Public API surface per crate |
| [Roadmap](roadmap.md) | Product backlog (Waves 0–4) and health checklist |
| [Features](features.md) | Future feature ideas (icebox) |
| [Review Prompts](review-prompts.md) | LLM review prompts for PRs |
