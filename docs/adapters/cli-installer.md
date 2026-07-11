# CLI Installer (`CliInstaller`)

Manages skill lifecycle — install, remove, update — by shelling out to `npx skills`.

## Port

```rust
impl SkillInstaller for CliInstaller {
    fn install(&self, name: &str, agents: &[String], scope: Scope)
        -> Result<(), Box<dyn std::error::Error>>;
    fn remove(&self, path: &Path)
        -> Result<(), Box<dyn std::error::Error>>;
    fn update(&self, path: &Path)
        -> Result<(), Box<dyn std::error::Error>>;
    fn preview_install(&self, ...) -> String;
    fn preview_remove(&self, ...) -> String;
    fn preview_update(&self, ...) -> String;
}
```

## Shell Commands

| Operation | Command |
|---|---|
| Install | `npx skills add <name> --<scope> [--agents <csv>]` |
| Remove | `npx skills remove <path>` |
| Update | `npx skills update <path>` |

### Scope Flag

```rust
fn scope_flag(scope: &Scope) -> &'static str {
    match scope {
        Scope::Global  => "--global",
        Scope::Project => "--project",
    }
}
```

### Agents Argument

```rust
fn agents_arg(agents: &[String]) -> Option<String> {
    if agents.is_empty() { None } else { Some(agents.join(",")) }
}
```

## Preview Methods

Preview methods return the command string **without executing** it. The TUI shows this in the confirmation dialog before the actual operation.

## Errors

```rust
pub enum CliInstallerError {
    Io(#[from] std::io::Error),   // Process spawn failure
    NonZeroExit(i32),             // npx returned non-zero
}
```

## Tests

- `preview_*` tests: verify command strings (no `npx` required)
- `live_install_and_remove`: `#[ignore = "requires npx"]`

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [NPX Catalog](npx-catalog.md) · [Toggler](toggler.md)
