# Testing Strategy

The project follows XP + TDD (strict Red → Green → Refactor). Every story in the backlog is labeled with the test level it requires.

## Test Levels

| Label | Level | Where | Characteristics |
|---|---|---|---|
| `[U]` | Unit | `core/` | Pure domain, no I/O, microsecond-fast |
| `[I]` | Integration | `adapters/` | Temp directories, fixtures, process execution |
| `[E]` | E2E | `tui/` | Render snapshots, state machine tests |

## Coverage by Level

### Unit Tests (`[U]`) — 78 tests in core

Every domain function is tested with edge cases:

- Frontmatter parsing: valid, missing delimiters, malformed YAML, frontmatter-only, full document
- Security scan: each pattern category, multiple findings, clean content, line numbers, case-insensitivity
- Profile diff: empty, identical, mixed install/remove, non-valid skill handling
- Duplicate detection: cross-scope, case-insensitive, three-way collisions
- Audit: single category, mixed state, empty input
- Drift state: equality, default, hashes exposure
- Skill model: field access, clone, scope inequality

### Integration Tests (`[I]`) — 49 tests in adapters

Tests use `tempfile::TempDir` for isolated filesystem operations:

- `FsSkillRepository`: empty roots, valid skills, broken symlinks, disabled dirs, duplicates, managed markers
- `FsToggler`: enable, disable, adopt, preview
- `FsProfileStore`: save, list, delete, sort, non-YAML filtering
- `FsWatcher`: creation event, empty paths, temp dir lifecycle
- `GitDriftChecker`: no-git, no-upstream
- `NpxCatalogGateway`: output parsing (empty, single, multiple, with URL)
- `CliInstaller`: preview outputs (install, remove, update, agents)
- `FsSkillCreator`: scaffold, path, frontmatter correctness

Tests requiring external tools are `#[ignore]`:
- `requires npx` — live `npx skills` calls
- `requires a git repo with a configured upstream remote`

### E2E Tests (`[E]`) — 132 tests in tui

Two categories:

**State machine** (~70 tests in `app::tests`):
- Key handlers for each view: navigation, multi-select, filters, actions
- Confirmation flow: pending action creation, acceptance, cancellation
- Wizard steps: Tab cycling, text input, scope toggling, scan gating
- State transitions: view switching, scroll, resize

**Render snapshots** (~62 tests in `ui::*.tests`):
- Each panel function rendered at least once with `ratatui::TestBackend`
- Snapshots using `insta` (YAML format)
- Edge cases: empty lists, error states, various validation states

## No `tests/` Directories

All tests are inline `#[cfg(test)] mod tests` within each source file (34 test modules across the workspace). This keeps tests close to the code they test and avoids visibility issues with private items.

## Snapshot Testing

```rust
#[test]
fn snapshot_valid_skill_with_content() {
    let skills = vec![valid_skill()];
    let mut buf = TestBackend::new(80, 24);
    let mut frame = Frame::new(&mut buf);
    render_detail_panel(&skills[0], 0, buf.area(), &mut frame);
    insta::assert_yaml_snapshot!(buf);
}
```

Snapshots are stored in `tui/src/ui/snapshots/` (managed by `insta`). Review with `cargo insta review` or accept with `cargo insta accept`.

---

[← Back to index](../index.md) · Related: [Setup](development.md) · [Release](release.md)
