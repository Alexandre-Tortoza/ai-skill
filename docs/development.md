# Development

## Setup

1. Install Rust via `rustup` or your system package manager. The expected toolchain is pinned in `rust-toolchain.toml`.
2. Clone the repository:
   ```sh
   git clone https://github.com/alexmrtr/ai-skill.git
   cd ai-skill
   ```
3. Build the workspace:
   ```sh
   cargo build --workspace
   ```

## Project Structure

```
ai-skill/
├── core/          # Pure domain logic, no I/O
├── adapters/      # I/O implementations (filesystem, shell, git)
├── tui/           # Terminal UI binary (ratatui + crossterm)
├── docs/          # Documentation
├── bin/           # Development scripts
└── .github/       # CI/CD workflows
```

### Architecture: Hexagonal

Dependency flow enforces a strict direction:

```
tui -> core <- adapters
```

- **`core`** (`ai-skill-core`): domain models, ports (traits), and pure functions. Has zero knowledge of filesystem, terminal, network, or external processes.
- **`adapters`** (`ai-skill-adapters`): implements `core`'s port traits with real I/O — filesystem scanning, shell commands (`npx skills`), git operations, filesystem watching via `notify`.
- **`tui`** (`ai-skill`): binary crate that wires adapters into `core` use cases and renders a `ratatui` interface.

`core` must never import `adapters`, `tui`, `ratatui`, `crossterm`, `notify`, or any I/O library.

## Commands

```sh
# Build everything
cargo build --workspace

# Run the TUI
cargo run -p ai-skill

# Run all tests
cargo test --workspace

# Format check
cargo fmt --check

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Security advisory check (if cargo-audit is installed)
cargo audit
```

## Methodology

This project follows **XP + TDD** (strict Red → Green → Refactor). Every functional change starts with a failing test.

- **`[U]`** — unit test in `core` (fast, no I/O)
- **`[I]`** — integration test in `adapters` (temp directories, fixtures)
- **`[E]`** — e2e / snapshot test in `tui` (render snapshots via `insta`)

Vertical slices are preferred over horizontal layers. Each story in `docs/roadmap.md` is a thin, testable slice.

## Scripts

The `bin/` directory contains helper scripts:

- **`bin/check`** — local quality gate: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and optionally `cargo audit`.
- **`bin/release-prep`** — pre-release validation: verifies manifest version matches the tag, confirms the `CHANGELOG.md` section exists, runs `bin/check`, and prints the next git steps.

## Testing Conventions

- `core` tests are pure unit tests — no temp files, no env mutation.
- `adapters` tests use `tempfile` for temporary directories and fixture directories under the crate.
- `tui` tests use `insta` for snapshot testing of rendered output.
- `unwrap()` is acceptable in tests. In production code it is reserved for cases where impossibility has been proven locally.

## CI/CD

- **CI** (`.github/workflows/ci.yml`): runs on every push and PR — `fmt`, `clippy`, `test`, `audit`.
- **Release** (`.github/workflows/release.yml`): triggered by tags `v*.*.*` — builds a matrix of 4 targets, publishes to crates.io, and creates a GitHub Release with checksums.

## Error Handling

- Boundary functions return `Result` with actionable error messages.
- User-facing errors should indicate a concrete next step (e.g., "install missing dependency", "fix path", "remove broken symlink", "review invalid frontmatter").
- Fail fast: if the system is in an unexpected state, surface it immediately rather than silently degrading.

## Conventions

- Every port in `core` models domain types — no leaky primitives.
- Prefer shelling out to external tooling (`npx skills`) over reimplementing remote APIs.
- The TUI must remain usable at 80×24 and respect `NO_COLOR`.

---

[← Back to index](index.md) · Related: [Testing](dev/testing.md) · [Release](dev/release.md) · [Crates](architecture/crates.md)
