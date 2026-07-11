# Drift Detection

Drift detection compares the local version of a skill against its upstream tracking branch. This lets users know when a skill has been updated upstream.

## Drift State

```rust
pub enum DriftState {
    Unknown,             // Not yet checked
    UpToDate,            // Local matches upstream
    UpdateAvailable {    // Local differs from upstream
        local_hash: String,
        upstream_hash: String,
    },
    NoGitRepo,           // Not inside a git repository
    NoUpstream,          // Git repo exists but no tracking branch
}
```

Default is `Unknown`. The TUI displays a yellow `[↑]` badge when `UpdateAvailable`.

## Drift Checker Port

```rust
pub trait DriftChecker {
    fn check(&self, path: &Path) -> DriftState;
}
```

The trait is object-safe. The `GitDriftChecker` adapter runs git commands to compute the state.

## How It Works

1. `GitDriftChecker::check(path)` is called with the skill's directory path
2. It runs `git rev-parse HEAD` to get the local hash
3. It runs `git rev-parse @{u}` to get the upstream tracking branch hash
4. If either command fails, returns `NoGitRepo` or `NoUpstream`
5. If hashes match, returns `UpToDate`
6. If hashes differ, returns `UpdateAvailable` with both hashes

## Limitations

- Requires the skill directory to be inside a git repository with a configured remote
- Detects any difference, not semantic version changes
- Does not show what changed (no diff viewer — planned for future)

---

[← Back to index](../index.md) · Related: [Skill Model](skill-model.md) · [Ports](ports.md) · [Audit](audit.md)
