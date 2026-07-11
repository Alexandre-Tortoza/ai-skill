# Release Process

## Trigger

Releases are triggered by pushing a tag matching `v*.*.*` or via `workflow_dispatch` in GitHub Actions.

## Pipeline

The release workflow (`.github/workflows/release.yml`) has three jobs:

### 1. Build (matrix)

Builds the `ai-skill` binary for 4 targets in parallel:

| Target | Platform | Runner |
|---|---|---|
| `x86_64-unknown-linux-gnu` | Linux x86_64 | `ubuntu-latest` |
| `x86_64-apple-darwin` | macOS Intel | `macos-latest` |
| `aarch64-apple-darwin` | macOS ARM | `macos-latest` |
| `x86_64-pc-windows-msvc` | Windows x86_64 | `windows-latest` |

Each job:
1. Checks out the tag
2. Builds release binary: `cargo build --release -p ai-skill`
3. Packages binary with `README.md`, `LICENSE`, `CHANGELOG.md`
4. Compresses: `.tar.gz` (Unix) or `.zip` (Windows)
5. Uploads as workflow artifact

ARM Linux builds use `cross` for cross-compilation (tests skipped — architecture mismatch).

### 2. Publish Crates

Publishes to crates.io in dependency order with 30-second delays:

```
ai-skill-core → ai-skill-adapters → ai-skill
```

Idempotent: checks if the version already exists on crates.io and skips if so (`grep "already exists"`).

### 3. Publish GitHub Release

1. Downloads all artifacts from the build job
2. Generates `SHA256SUMS` for every artifact
3. Extracts the changelog section for this version from `CHANGELOG.md`
4. Creates a GitHub Release with:
   - Title: version tag
   - Body: extracted changelog section + installation instructions
   - Attachments: all archive files + checksums

## Pre-flight Script

```sh
bin/release-prep v0.2.0
```

Validates:
- Manifest version matches the tag
- `CHANGELOG.md` has a section for the version
- Repository is clean (no uncommitted changes)
- Runs `bin/check` (fmt, clippy, test)
- Prints next git steps

## Manual Steps

```sh
# Update version
cargo set-version 0.2.0

# Update CHANGELOG.md with the new version section

# Commit and tag
git add .
git commit -m "release: v0.2.0"
git tag v0.2.0

# Push
git push origin main
git push origin v0.2.0
```

## CI Quality Gate

Every push and PR runs:

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo audit
```

Cached with `Swatinem/rust-cache@v2` to cut build time.

## Versioning

Follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Changelog follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format.

---

[← Back to index](../index.md) · Related: [Testing](testing.md) · [Development](../development.md)
