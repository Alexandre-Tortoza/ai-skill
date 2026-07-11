# Profiles

Profiles are named sets of skills that can be activated as a group. The system computes a diff between the current state and the desired profile, executing only the minimal operations needed.

## Domain Types

### `Profile`

```rust
pub struct Profile {
    pub name: String,
    pub skill_names: Vec<String>,
}
```

A named collection of skill names. Serialized/deserialized via serde (YAML persistence).

### `ProfileOp`

```rust
pub enum ProfileOp {
    Install { name: String },
    Remove { name: String },
}
```

A single operation to reconcile current state with a profile.

## Diff Algorithm

```rust
pub fn diff_profile(current: &[Skill], desired: &Profile) -> Vec<ProfileOp>
```

**Logic:**
1. Consider only `ValidationState::Valid` skills as "installed"
2. Skills in `desired.skill_names` but not in current valid skills → `Install`
3. Skills in current valid skills but not in `desired.skill_names` → `Remove`
4. Non-valid skills are excluded from removal (avoids cascading errors)

**Example:**

```rust
let current = vec![skill_a, skill_b];  // both Valid
let desired = Profile { name: "dev".into(), skill_names: vec!["b".into(), "c".into()] };
let ops = diff_profile(&current, &desired);
// Result: [Remove { name: "a" }, Install { name: "c" }]
```

## Profile Store Port

```rust
pub trait ProfileStore {
    fn list(&self) -> Result<Vec<Profile>, Box<dyn std::error::Error>>;
    fn save(&self, profile: &Profile) -> Result<(), Box<dyn std::error::Error>>;
    fn delete(&self, name: &str) -> Result<(), Box<dyn std::error::Error>>;
}
```

Object-safe. Implemented by `FsProfileStore` which persists profiles as YAML files in `~/.claude/ai-skill/profiles/`.

## TUI Integration

The Profiles panel in the TUI provides:
- **List profiles** with skill count
- **View profile detail** — skill names in the selected profile
- **Create profile** from current installed state
- **Activate profile** — computes diff and executes batch operations
- **Delete profile**

---

[← Back to index](../index.md) · Related: [Ports](ports.md) · [Skill Model](skill-model.md)
