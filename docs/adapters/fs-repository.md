# FS Repository (`FsSkillRepository`)

Scans the filesystem for installed skills, validates each one, and populates `ValidationState`.

## Port

```rust
impl SkillRepository for FsSkillRepository {
    type Error = RepositoryError;
    fn list(&self) -> Result<Vec<Skill>, RepositoryError>;
}
```

## Structure

```rust
pub struct FsSkillRepository {
    global_root: PathBuf,          // ~/.claude/skills
    project_root: Option<PathBuf>, // $PWD/.claude/skills (if exists)
}
```

## Scanning Logic

`list()` scans both roots and returns a flat list of `Skill` objects:

1. **Discover directories**: iterate each entry in the root directory
2. **Scope assignment**: global root → `Scope::Global`, project root → `Scope::Project`
3. **Symlink resolution**: resolve `Path::canonicalize` — failure → `BrokenSymlink`
4. **Manifest check**: look for `SKILL.md` — missing → `MissingManifest`
5. **Frontmatter parsing**: parse `SKILL.md` — failure → `InvalidFrontmatter`
6. **Disabled detection**: directory name ends with `.disabled` → `Disabled`
7. **Managed detection**: `.ai-skill` file present → `managed = true`
8. **Orphan lock**: lock file without skill directory → `OrphanLock` (stub: always `false`)
9. **Duplicate detection**: cross-scope case-insensitive name collisions → `Duplicate`

## Constructor

```rust
pub fn new(global_root: PathBuf, project_root: Option<PathBuf>) -> Self
pub fn from_env() -> Result<Self, RepositoryError>
```

`from_env()` resolves:
- **Global**: `$HOME/.claude/skills`
- **Project**: `$PWD/.claude/skills` (only if the directory exists)

## Error Handling

```rust
pub enum RepositoryError {
    Io(#[from] std::io::Error),
}
```

I/O errors during scan are propagated. Individual skill failures (e.g., broken symlink) are captured in `ValidationState`, not returned as errors.

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [FS Skill Creator](fs-skill-creator.md) · [Toggler](toggler.md) · [Watcher](watcher.md)
