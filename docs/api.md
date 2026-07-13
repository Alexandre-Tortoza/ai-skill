# API Reference

`ai-skill` is a Rust workspace with three crates. Below is the public API surface of each crate.

## `ai-skill-core`

Pure domain layer. No I/O dependencies.

### Traits (Ports)

| Trait | Primary method | Purpose |
|---|---|---|
| `SkillRepository` | `list()` | List all installed skills |
| `AnyCatalogGateway` | `search(keyword)` | Search remote skill catalog |
| `SkillInstaller` | `install()`, `remove()`, `update()` | Manage skill lifecycle |
| `SkillToggler` | `enable()`, `disable()`, `adopt()` | Toggle skill state |
| `ProfileStore` | `list()`, `save()`, `delete()` | Persist named profiles |
| `DriftChecker` | `check(path)` | Detect upstream drift |
| `SkillDiffReader` | `read_diff(path)` | Read upstream diff of a skill's manifest |
| `SkillCreator` | `create(name, agents, tags)` | Scaffold new skill |
| `SkillUsageReader` | `read_events()` | Read local agent usage history |
| `SkillWriter` | `write(path, content)` | Write SKILL.md to disk |

### Domain Structs

| Struct | Fields |
|---|---|
| `Skill` | `name`, `path`, `scope`, `agents`, `tags`, `managed`, `validation`, `manifest_content`, `drift_state` |
| `CatalogEntry` | `name`, `description`, `url` |
| `Profile` | `name`, `skill_names` |
| `AuditReport` | `broken`, `duplicates`, `no_agents`, `update_available` |
| `SkillUsageEvent` | `skill_name`, `timestamp` |
| `UsageReport` | `records`, `dead`, `stale`, `stale_after_days` |
| `SkillMetadata` | `name`, `agents`, `tags` |
| `ScanFinding` | `severity`, `category`, `detail`, `line` |
| `ThemeSlot` | `error`, `warning`, `success`, `accent`, `muted`, `dead`, `stale` |
| `Theme` | `color(slot)` — resolved semantic palette |
| `KeyBindings` | `matches(key, action)`, `from_config(map)` |
| `SkillDiff` | `lines: Vec<DiffLine>` |
| `DiffLine` | `kind: DiffLineKind`, `text: String` |
| `Action` | `quit`, `help`, `audit`, `search`, `create`, `profiles`, `bundles`, `budget`, `editor`, `sync`, `ssh`, `adopt`, `toggle_name_only`, `disable`, `enable`, `remove`, `update` |

### Domain Enums

| Enum | Variants |
|---|---|
| `Scope` | `Global`, `Project` |
| `ValidationState` | `Valid`, `BrokenSymlink`, `MissingManifest`, `InvalidFrontmatter { reason }`, `OrphanLock`, `Duplicate { conflicts_with }`, `Disabled` |
| `DriftState` | `Unknown`, `UpToDate`, `UpdateAvailable { local_hash, upstream_hash }`, `NoGitRepo`, `NoUpstream` |
| `DiffLineKind` | `Context`, `Add`, `Remove`, `Header` |
| `DiffError` | `NoGitRepo`, `NoUpstream`, `CommandFailed` |
| `ProfileOp` | `Install { name }`, `Remove { name }` |
| `Severity` | `High`, `Medium` |
| `ScanCategory` | `DangerousShellPattern`, `EnvVarHarvest`, `HardcodedSecret`, `PromptInjection` |
| `ParseError` | `MissingDelimiters`, `Yaml` |

### Free Functions

| Function | Signature |
|---|---|
| `audit_skills` | `(skills: &[Skill]) -> AuditReport` |
| `build_usage_report` | `(events: &[SkillUsageEvent], skill_names: &[String], stale_after_days: u64) -> UsageReport` |
| `detect_duplicates` | `(skills: &[Skill]) -> Vec<(usize, PathBuf)>` |
| `diff_profile` | `(current: &[String], desired: &[String]) -> Vec<ProfileOp>` |
| `parse_diff` | `(raw: &str) -> SkillDiff` |
| `parse_frontmatter` | `(content: &str) -> Result<SkillMetadata, ParseError>` |
| `extract_body` | `(content: &str) -> Option<&str>` |
| `scan_skill` | `(content: &str) -> Vec<ScanFinding>` |
| `scaffold_skill` | `(name: &str, agents: &[String], tags: &[String]) -> String` |
| `apply_edit` | `(original: &str, name: &str, agents: &[String], tags: &[String]) -> String` |

## `ai-skill-adapters`

I/O implementations of core ports.

### Adapter Structs

| Struct | Implements | Purpose |
|---|---|---|
| `FsSkillRepository` | `SkillRepository` | Scans `~/.claude/skills` and project `.claude/skills` |
| `NpxCatalogGateway` | `AnyCatalogGateway` | Shells out to `npx skills find` |
| `CliInstaller` | `SkillInstaller` | Shells out to `npx skills add/remove/update` |
| `FsToggler` | `SkillToggler` | Renames directories (`.disabled` suffix, `.ai-skill` marker) |
| `FsProfileStore` | `ProfileStore` | Reads/writes YAML in `~/.claude/ai-skill/profiles/` |
| `GitDriftChecker` | `DriftChecker` | Runs `git rev-parse` commands |
| `GitSkillDiffReader` | `SkillDiffReader` | Shells out to `git diff HEAD..@{u} -- SKILL.md` |
| `FsSkillCreator` | `SkillCreator` | Creates skill directories with scaffolds |
| `FsSkillWriter` | `SkillWriter` | Writes SKILL.md files |
| `FsWatcher` | (none) | Debounced filesystem watcher via `notify` |
| `FsUsageHistoryReader` | `SkillUsageReader` | Scans Claude Code `.jsonl` transcripts for skill usage |

### Constructor Conventions

Each adapter provides:
- **`new(...)`** — takes explicit paths/parameters for testability.
- **`from_env()`** — resolves default paths from environment / home directory.

### Error Types

| Error | Description |
|---|---|
| `RepositoryError(Io)` | Filesystem I/O error |
| `CliInstallerError(Io | NonZeroExit)` | Process spawn or non-zero exit |

## `ai-skill`

Binary crate — no public API. Internal architecture:

- **`App<G, I, T>`** — generic over gateway, installer, toggler. Holds all state, dispatches to 12 view handlers.
- **`View`** — enum with 13 variants mapping to UI panels (incl. `List` split preview and `Explorer`).
- **`AppEvent`** — `Key(KeyEvent)` or `Resize(u16, u16)`.
- **`terminal`** module — `setup()`, `teardown()`, `install_panic_hook()` for crossterm lifecycle.
- **`ui`** module — submodules, one per panel/widget, plus:
  - **`theme`** — `Theme` (semantic colour slots) and `parse_color` for customizing the palette via `config.json`.
  - **`keymap`** — `Action` enum and `KeyBindings` for customizable shortcuts (resolved from `config.json`).
   - **`i18n`** — `Locale` (en / pt-BR) and `I18n` for localized UI strings, resolved from `config.json` `locale`. `I18n::from_config(None)` falls back to English.
   - **`diff_panel`** — `render_diff_panel(...)`: color-coded renderer for a skill's upstream diff (`SkillDiff`), reached from the detail view via `d` when an update is available.
    - **`command_palette`** — `render_command_palette(...)`: floating overlay listing `PaletteCommand`s (`Search`, `Create`, `Audit`, `Budget`, `Profiles`, `Bundles`, `Sync`, `Settings`, `Help`, and selected-skill actions `OpenDetail`/`Edit`/`Disable`/`Remove`/`Update`/`Diff`). Opened with `Ctrl+P`; `Enter` runs the selected command.
    - **`split_preview_panel`** — `render_split_preview(...)`: List view split — `installed_panel` (left) plus a live `README.md`/`SKILL.md` preview (right) for the selected skill.
    - **`skill_explorer_panel`** — `render_skill_explorer(...)`: `Enter` from the list opens the directory tree (left) and file content (right) for the selected skill; nested sub-skills are marked `◈`.
   - **Quit** — `Ctrl+C` must be pressed twice within 3s (`App::request_quit` arms a timer; `quit_warning_active` drives the status-bar warning). `q` no longer quits; `Esc` only closes panels/modals.

The binary entry point in `main.rs` wires real adapters and runs the ratatui event loop.

---

[← Back to index](index.md) · Related: [Ports](core/ports.md) · [Overview](architecture/overview.md)
