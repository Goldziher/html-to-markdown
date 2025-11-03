#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "Usage: $0 <version> <output-dir>" >&2
  exit 1
fi

VERSION="$1"
DEST_DIR="$2"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGING="$(mktemp -d "${ROOT}/.pie-src.XXXXXX")"

cleanup() {
  rm -rf "$STAGING"
}
trap cleanup EXIT

mkdir -p "$DEST_DIR"

WORKSPACE_DIR="$STAGING/workspace"
mkdir -p "$WORKSPACE_DIR"

# Base files required to build the Rust workspace
cp "$ROOT/Cargo.toml" "$WORKSPACE_DIR/"
if [[ -f "$ROOT/Cargo.lock" ]]; then
  cp "$ROOT/Cargo.lock" "$WORKSPACE_DIR/"
fi
cp "$ROOT/LICENSE" "$STAGING/"
cp "$ROOT/README.md" "$STAGING/PROJECT-README.md"

rsync -a --exclude 'target' --exclude 'debug' "$ROOT/crates" "$WORKSPACE_DIR/"

# PIE metadata lives under packages/php-ext; copy into staging root.
cp "$ROOT/composer.json" "$STAGING/"
cp "$ROOT/packages/php-ext/config.m4" "$STAGING/"
cp "$ROOT/packages/php-ext/Makefile.frag" "$STAGING/"
cp "$ROOT/packages/php-ext/Makefile.frag.w32" "$STAGING/"
cp "$ROOT/packages/php-ext/config.w32" "$STAGING/"
cp "$ROOT/packages/php-ext/README.md" "$STAGING/PIE-README.md"

# Include helper scripts needed during the build (if any)
if [[ -d "$ROOT/packages/php-ext/bin" ]]; then
  rsync -a "$ROOT/packages/php-ext/bin" "$STAGING/"
fi

ARCHIVE_NAME="php_html_to_markdown-${VERSION}-src.tgz"
tar -czf "${DEST_DIR}/${ARCHIVE_NAME}" -C "$STAGING" .

echo "Created ${DEST_DIR}/${ARCHIVE_NAME}"
