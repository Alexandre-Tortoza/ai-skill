# Toggler (`FsToggler`)

Enables, disables, and adopts skills via filesystem operations.

## Port

```rust
impl SkillToggler for FsToggler {
    fn enable(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    fn disable(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    fn adopt(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>;
    fn preview_enable(&self, path: &Path) -> String;
    fn preview_disable(&self, path: &Path) -> String;
}
```

## Operations

### Disable

Renames the skill directory to append `.disabled`:

```
my-skill/ → my-skill.disabled/
```

Agents ignore `.disabled` directories. This is the standard Claude Code convention for disabling skills.

### Enable

Renames back by removing the `.disabled` suffix:

```
my-skill.disabled/ → my-skill/
```

### Adopt

Creates a `.ai-skill` marker file inside the skill directory:

```
touch my-skill/.ai-skill
```

This marks the skill as managed by `ai-skill` for status display and filtering. The marker is an empty file — its existence is all that matters.

## Preview Methods

Return human-readable descriptions without executing:

- `preview_enable("my-skill.disabled")` → `"Enable: my-skill.disabled → my-skill"`
- `preview_disable("my-skill")` → `"Disable: my-skill → my-skill.disabled"`

## Edge Cases

| Scenario | Behavior |
|---|---|
| `enable` on path without `.disabled` | No-op (returns `Ok`) |
| `disable` on path already with `.disabled` | Appends another `.disabled` (no special handling) |
| `adopt` on non-existent directory | File write error propagated |

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [FS Repository](fs-repository.md) · [CLI Installer](cli-installer.md)
