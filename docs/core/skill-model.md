# Skill Model

## `Skill` — Central Domain Struct

```rust
pub struct Skill {
    pub name: String,
    pub path: PathBuf,
    pub scope: Scope,
    pub agents: Vec<String>,
    pub tags: Vec<String>,
    pub managed: bool,
    pub validation: ValidationState,
    pub manifest_content: Option<String>,
    pub drift_state: DriftState,
}
```

| Field | Source | Meaning |
|---|---|---|
| `name` | `SKILL.md` frontmatter | Display name |
| `path` | Filesystem | Resolved canonical path |
| `scope` | Directory location | Global or project-level |
| `agents` | Frontmatter | Target AI agents |
| `tags` | Frontmatter | Classification tags |
| `managed` | `.ai-skill` marker | Tracked by ai-skill |
| `validation` | Scan result | Current health state |
| `manifest_content` | `SKILL.md` content | Raw manifest text |
| `drift_state` | Git comparison | Upstream sync state |

## `Scope` — Installation Scope

```rust
pub enum Scope {
    Global,   // ~/.claude/skills/
    Project,  // ./.claude/skills/
}
```

`Default` is `Global`. The scope determines which directory the skill lives in and affects filtering in the TUI.

## `ValidationState` — Skill Health

```rust
pub enum ValidationState {
    Valid,                               // Everything OK
    BrokenSymlink,                       // Symlink target missing
    MissingManifest,                     // No SKILL.md found
    InvalidFrontmatter { reason: String }, // YAML parse error
    OrphanLock,                          // Lock file without skill (stub)
    Duplicate { conflicts_with: PathBuf }, // Name collision
    Disabled,                            // .disabled suffix present
}
```

The `FsSkillRepository` populates this during scanning. Each variant maps to a colored badge in the TUI.

## `DriftState` — Upstream Sync

```rust
pub enum DriftState {
    Unknown,
    UpToDate,
    UpdateAvailable { local_hash: String, upstream_hash: String },
    NoGitRepo,
    NoUpstream,
}
```

`Default` is `Unknown`. The `GitDriftChecker` populates this by comparing git tree hashes.

---

[← Back to index](../index.md) · Related: [Ports](ports.md) · [Duplicate Detection](duplicate-detector.md) · [Frontmatter](frontmatter.md) · [Drift Detection](drift.md)
