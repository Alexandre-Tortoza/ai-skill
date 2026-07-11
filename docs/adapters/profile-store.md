# Profile Store (`FsProfileStore`)

Persists named skill profiles as YAML files on the filesystem.

## Port

```rust
impl ProfileStore for FsProfileStore {
    fn list(&self) -> Result<Vec<Profile>, Box<dyn std::error::Error>>;
    fn save(&self, profile: &Profile) -> Result<(), Box<dyn std::error::Error>>;
    fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error>>;
}
```

## Structure

```rust
pub struct FsProfileStore {
    base_dir: PathBuf,
}
```

Default path: `$HOME/.claude/ai-skill/profiles/`

## Persistence Format

Each profile is stored as a separate YAML file:

```
~/.claude/ai-skill/profiles/
├── dev.yaml
├── test.yaml
└── release.yaml
```

### Example `dev.yaml`

```yaml
name: dev
skill_names:
  - rust-analyzer
  - eslint
  - prettier
```

## Operations

### `list()`

- Reads all `*.yaml` files from `base_dir`
- Returns profiles sorted by name (case-insensitive)
- Returns empty `Vec` if directory doesn't exist

### `save()`

- Creates `base_dir` if it doesn't exist
- Serializes profile as YAML
- Writes to `{base_dir}/{name}.yaml`

### `delete()`

- Removes `{base_dir}/{name}.yaml`
- No-op if file doesn't exist

## Internal

```rust
fn profile_path(&self, name: &str) -> PathBuf {
    self.base_dir.join(format!("{name}.yaml"))
}
```

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [FS Repository](fs-repository.md)
