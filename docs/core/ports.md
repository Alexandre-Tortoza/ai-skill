# Port Traits

All port traits live in `core` and define the boundary between domain logic and I/O. Adapters in the `adapters` crate provide concrete implementations.

## `SkillRepository`

```rust
pub trait SkillRepository {
    type Error: std::error::Error;
    fn list(&self) -> Result<Vec<Skill>, Self::Error>;
}
```

The fundamental port: list all installed skills. Returns domain `Skill` objects fully populated with validation state and drift information.

**Adapter:** `FsSkillRepository`

## `AnyCatalogGateway`

```rust
pub trait AnyCatalogGateway {
    fn search(&self, keyword: &str)
        -> Result<Vec<CatalogEntry>, Box<dyn std::error::Error>>;
}
```

Remote catalog search. Returns entries with name, description, and optional URL. Object-safe (used as `Box<dyn AnyCatalogGateway>`).

**Adapter:** `NpxCatalogGateway`

## `SkillInstaller`

```rust
pub trait SkillInstaller {
    fn install(&self, name: &str, agents: &[String], scope: Scope)
        -> Result<(), Box<dyn std::error::Error>>;
    fn remove(&self, path: &Path)
        -> Result<(), Box<dyn std::error::Error>>;
    fn update(&self, path: &Path)
        -> Result<(), Box<dyn std::error::Error>>;
    fn preview_install(&self, name: &str, agents: &[String], scope: Scope) -> String;
    fn preview_remove(&self, path: &Path) -> String;
    fn preview_update(&self, path: &Path) -> String;
}
```

Lifecycle management for skills. Preview methods return a human-readable string describing the operation without executing it.

**Adapter:** `CliInstaller`

## `SkillToggler`

```rust
pub trait SkillToggler {
    fn enable(&self, path: &Path)
        -> Result<(), Box<dyn std::error::Error>>;
    fn disable(&self, path: &Path)
        -> Result<(), Box<dyn std::error::Error>>;
    fn adopt(&self, path: &Path)
        -> Result<(), Box<dyn std::error::Error>>;
    fn preview_enable(&self, path: &Path) -> String;
    fn preview_disable(&self, path: &Path) -> String;
}
```

Toggle skill state: enable (remove `.disabled` suffix), disable (add `.disabled` suffix), adopt (create `.ai-skill` marker). Object-safe.

**Adapter:** `FsToggler`

## `ProfileStore`

```rust
pub trait ProfileStore {
    fn list(&self) -> Result<Vec<Profile>, Box<dyn std::error::Error>>;
    fn save(&self, profile: &Profile)
        -> Result<(), Box<dyn std::error::Error>>;
    fn delete(&self, name: &str)
        -> Result<(), Box<dyn std::error::Error>>;
}
```

Persist named skill profiles. Object-safe.

**Adapter:** `FsProfileStore`

## `DriftChecker`

```rust
pub trait DriftChecker {
    fn check(&self, path: &Path) -> DriftState;
}
```

Check whether a skill's local version differs from its upstream tracking branch. Object-safe.

**Adapter:** `GitDriftChecker`

## `SkillCreator`

```rust
pub trait SkillCreator {
    fn create(&self, name: &str, agents: &[String], tags: &[String])
        -> Result<PathBuf, Box<dyn std::error::Error>>;
}
```

Create a new skill directory with scaffolded `SKILL.md`. Returns the created directory path. Object-safe.

**Adapter:** `FsSkillCreator`

## `SkillWriter`

```rust
pub trait SkillWriter {
    fn write(&self, path: &Path, content: &str)
        -> Result<(), Box<dyn std::error::Error>>;
}
```

Write content to a file at the given path. Separate from `SkillCreator` to support the editor use case (edit existing skill content). Object-safe.

**Adapter:** `FsSkillWriter`

## `SkillContentReader`

```rust
pub trait SkillContentReader {
    fn read_preview(&self, skill_dir: &Path) -> Result<SkillDoc, ContentError>;
    fn read_tree(&self, skill_dir: &Path) -> Result<Vec<SkillTreeNode>, ContentError>;
    fn read_file(&self, file_path: &Path) -> Result<String, ContentError>;
}
```

Reads a skill's on-disk file content for the TUI split preview and directory explorer.
`read_preview` resolves `README.md` → `readme.md` → `Readme.md` → `SKILL.md` (stripping
frontmatter for `SKILL.md`); `read_tree` returns a depth-first listing that flags nested
sub-skills (directories containing `SKILL.md`); `read_file` returns raw text capped at 64 KB.
Object-safe.

**Adapter:** `FsSkillContentReader`

---

[← Back to index](../index.md) · Related: [Skill Model](skill-model.md) · [Profiles](profiles.md) · [Drift Detection](drift.md)
