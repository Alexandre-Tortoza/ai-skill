#!/usr/bin/env sh
set -eu

repo="alexmrtr/ai-skill"
base_url="https://github.com/${repo}/releases/latest/download"
bin_dir="${BIN_DIR:-$HOME/.local/bin}"
dry_run=0

usage() {
  cat <<EOF
usage: install.sh [--dry-run]

Environment:
  BIN_DIR  Installation directory. Defaults to \$HOME/.local/bin.
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run)
      dry_run=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "required command not found: $1" >&2
    exit 1
  fi
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os:$arch" in
    Linux:x86_64|Linux:amd64)
      echo "x86_64-unknown-linux-gnu"
      ;;
    Darwin:x86_64|Darwin:amd64)
      echo "x86_64-apple-darwin"
      ;;
    Darwin:arm64|Darwin:aarch64)
      echo "aarch64-apple-darwin"
      ;;
    *)
      cat >&2 <<EOF
unsupported platform: $os $arch

Supported by this installer:
  Linux x86_64
  macOS x86_64
  macOS arm64

Windows users should download the .zip from GitHub Releases manually.
EOF
      exit 1
      ;;
  esac
}

verify_checksum() {
  asset="$1"
  checksums="$2"

  awk -v asset="$asset" '$2 == asset { print }' "$checksums" > "$checksums.one"
  if [ ! -s "$checksums.one" ]; then
    echo "checksum for $asset not found in SHA256SUMS" >&2
    exit 1
  fi

  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -c "$checksums.one"
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -c "$checksums.one"
  else
    echo "required command not found: sha256sum or shasum" >&2
    exit 1
  fi
}

target="$(detect_target)"
asset="ai-skill-${target}.tar.gz"
archive_url="${base_url}/${asset}"
checksums_url="${base_url}/SHA256SUMS"

if [ "$dry_run" -eq 1 ]; then
  cat <<EOF
Would install ai-skill:
  target:   $target
  asset:    $asset
  archive:  $archive_url
  checksum: $checksums_url
  bin dir:  $bin_dir
EOF
  exit 0
fi

need curl
need tar
need mkdir
need mktemp

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

echo "Downloading $asset"
curl -fsSL "$archive_url" -o "$tmp/$asset"
curl -fsSL "$checksums_url" -o "$tmp/SHA256SUMS"

cd "$tmp"
verify_checksum "$asset" "SHA256SUMS"
tar -xzf "$asset"

mkdir -p "$bin_dir"
cp "ai-skill-${target}/ai-skill" "$bin_dir/ai-skill"
chmod 755 "$bin_dir/ai-skill"

cat <<EOF
ai-skill installed to $bin_dir/ai-skill

If this directory is not on your PATH, add it before running ai-skill:
  export PATH="$bin_dir:\$PATH"
EOF
