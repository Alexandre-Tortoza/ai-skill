# Git Drift Checker (`GitDriftChecker`)

Checks whether a skill's local version has drifted from its upstream tracking branch.

## Port

```rust
impl DriftChecker for GitDriftChecker {
    fn check(&self, path: &Path) -> DriftState;
}
```

Unlike other adapters, `check` returns `DriftState` directly instead of `Result`. Git failures are captured as enum variants (`NoGitRepo`, `NoUpstream`) rather than errors.

## How It Works

```rust
fn run_git(path: &Path, args: &[&str]) -> Option<String> {
    Command::new("git")
        .args(args)
        .current_dir(path)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}
```

### Steps

1. Run `git rev-parse HEAD` — if this fails, the directory is not a git repo → `NoGitRepo`
2. Run `git rev-parse @{u}` — if this fails, no upstream configured → `NoUpstream`
3. Compare hashes:
   - Equal → `UpToDate`
   - Different → `UpdateAvailable { local_hash, upstream_hash }`

## Limitations

- Requires the skill directory to be inside a git repository
- Requires a configured upstream tracking branch
- Detects any difference, not a semantic version bump
- Returns `NoGitRepo` for directories inside a repo where `git` command fails at the directory level (e.g., submodule issues)

## Tests

- `path_without_git_returns_no_git_repo`: uses a temp directory, no git init
- `git_repo_without_upstream_returns_no_upstream`: inits a git repo without remote
- `live_repo_with_upstream_returns_*`: `#[ignore = "requires a git repo with a configured upstream remote"]`

---

[← Back to index](../index.md) · Related: [Overview](overview.md) · [FS Repository](fs-repository.md)
