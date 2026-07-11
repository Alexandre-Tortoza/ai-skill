# Installation

## Prerequisites

- **Rust toolchain** (if building from source): version `1.95` or later, pinned in [`rust-toolchain.toml`](../rust-toolchain.toml).
- **npx** (Node.js): required for remote catalog search and skill lifecycle operations (`npx skills find`, `npx skills add`, etc.).

## Download a Release

Pre-built binaries are published for each [GitHub Release](https://github.com/alexmrtr/ai-skill/releases).

| Platform | Archive |
|---|---|
| Linux x86_64 | `ai-skill-x86_64-unknown-linux-gnu.tar.gz` |
| macOS Intel | `ai-skill-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `ai-skill-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `ai-skill-x86_64-pc-windows-msvc.zip` |

### Steps

1. Download the archive for your platform from the latest release.
2. Verify the checksum:
   ```sh
   sha256sum -c SHA256SUMS 2>/dev/null | grep OK
   ```
3. Extract and move the binary to a directory in your `PATH`:
   ```sh
   tar xzf ai-skill-*.tar.gz
   sudo mv ai-skill /usr/local/bin/
   ```

### Platform notes

- **macOS**: Binaries are not yet signed or notarized. If Gatekeeper blocks execution, verify the source and checksum, then remove the quarantine attribute:
  ```sh
  xattr -d com.apple.quarantine ./ai-skill
  ```
- **Windows**: Binaries are not yet signed. SmartScreen may show a warning on first run.

## Build from Source

```sh
git clone https://github.com/alexmrtr/ai-skill.git
cd ai-skill
cargo build --release -p ai-skill
```

The binary is placed at `target/release/ai-skill`.

## Verify

```sh
ai-skill --help
```

If the help text appears, the installation is complete.

---

[← Back to index](index.md) · Related: [Usage](usage.md) · [Security](security.md)
