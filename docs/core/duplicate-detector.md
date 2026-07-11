# Duplicate Detection

Detects case-insensitive name collisions across scopes (global vs. project).

## Public API

```rust
pub fn detect_duplicates(skills: &[Skill]) -> Vec<(usize, PathBuf)>
```

Returns a list of `(index, path_of_first_occurrence)` for every skill that has a duplicate name.

## Algorithm

1. Iterate through all skills
2. Build a map of lowercase name → (first_index, first_path)
3. For each skill whose lowercase name already exists in the map, add its index and the first occurrence's path to the result
4. The first occurrence of each name is never flagged as a duplicate

## Example

```rust
let skills = vec![
    Skill { name: "lint".into(), path: "global/lint", scope: Global, .. },
    Skill { name: "Lint".into(), path: "project/Lint", scope: Project, .. },
    Skill { name: "test".into(), path: "global/test", scope: Global, .. },
];
let duplicates = detect_duplicates(&skills);
// Result: [(1, PathBuf::from("global/lint"))]
// "lint" and "Lint" collide (case-insensitive)
// index 0 (first occurrence) is not flagged
// index 1 (second occurrence) is flagged with path of index 0
// "test" has no collision
```

## Edge Cases

| Scenario | Behavior |
|---|---|
| No duplicates | Returns empty `Vec` |
| First occurrence | Never flagged |
| Three same names (a, A, a) | Index 1 and 2 flagged, index 0 not flagged |
| Case-insensitive (Lint, LINT, lint) | All collisions detected |

## Integration

`detect_duplicates` is called by `FsSkillRepository::list()` during scanning. The resulting `Duplicate { conflicts_with }` state is set on the flagged skills. The first occurrence keeps its original validation state (typically `Valid`).

---

[← Back to index](../index.md) · Related: [Audit](audit.md) · [Skill Model](skill-model.md) · [FS Repository](../adapters/fs-repository.md)
