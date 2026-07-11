# Audit System

The audit system produces an aggregated report across all installed skills, categorizing them by health status.

## Audit Report

```rust
pub struct AuditReport<'a> {
    pub broken: Vec<&'a Skill>,       // BrokenSymlink, MissingManifest, InvalidFrontmatter, OrphanLock
    pub duplicates: Vec<&'a Skill>,   // Duplicate
    pub no_agents: Vec<&'a Skill>,    // Valid or Disabled with empty agents
    pub update_available: Vec<&'a Skill>,  // DriftState::UpdateAvailable
}
```

The report borrows from the skills slice — zero allocation for the classification itself.

## Classification Logic

```rust
pub fn audit_skills(skills: &[Skill]) -> AuditReport<'_>
```

| Category | Included states |
|---|---|
| `broken` | `BrokenSymlink`, `MissingManifest`, `InvalidFrontmatter`, `OrphanLock` |
| `duplicates` | `Duplicate` |
| `no_agents` | `Valid` or `Disabled` with empty `agents` vector |
| `update_available` | Any skill with `DriftState::UpdateAvailable` |

## Duplicate Detection

```rust
pub fn detect_duplicates(skills: &[Skill]) -> Vec<(usize, PathBuf)>
```

Returns `(index, path_of_first_occurrence)` for each duplicate. Detection is case-insensitive on `skill.name`. The first occurrence of each name is not included in the output.

## TUI Integration

The Audit panel renders four sections with color-coded status:

| Section | Color | Content |
|---|---|---|
| Broken | Red | Skills with broken symlinks or missing manifests |
| Duplicates | Cyan | Skills with name collisions |
| No Agents | Yellow | Skills without target agents |
| Updates | Green | Skills with upstream updates available |

---

[← Back to index](../index.md) · Related: [Skill Model](skill-model.md) · [Duplicate Detection](duplicate-detector.md) · [Drift Detection](drift.md) · [Frontmatter](frontmatter.md)
