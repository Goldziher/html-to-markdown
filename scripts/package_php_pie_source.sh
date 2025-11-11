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

EXT_SRC_DIR="$ROOT/packages/php-ext"

# Copy extension scaffold (config.m4, sources, etc.) to the archive root so PIE builds in place.
rsync -a "$EXT_SRC_DIR"/ "$STAGING"/
# Preserve legacy layout under packages/php-ext for consumers expecting that path.
mkdir -p "$STAGING/packages/php-ext"
rsync -a "$EXT_SRC_DIR"/ "$STAGING/packages/php-ext"/

WORKSPACE_DIR="$STAGING/workspace"
mkdir -p "$WORKSPACE_DIR"

# Base files required to build the Rust workspace
cp "$ROOT/Cargo.toml" "$WORKSPACE_DIR/"
if [[ -f "$ROOT/Cargo.lock" ]]; then
  cp "$ROOT/Cargo.lock" "$WORKSPACE_DIR/"
fi
cp "$ROOT/LICENSE" "$STAGING/"
cp "$ROOT/README.md" "$STAGING/PROJECT-README.md"

# Add all Rust workspace crates plus tooling needed by the workspace members.
rsync -a --exclude 'target' --exclude 'debug' "$ROOT/crates" "$WORKSPACE_DIR/"

# Copy workspace tools (e.g., runtime benchmarks) so cargo metadata stays intact.
if [[ -d "$ROOT/tools" ]]; then
  mkdir -p "$WORKSPACE_DIR/tools"
  rsync -a --exclude 'target' --exclude 'debug' "$ROOT/tools/" "$WORKSPACE_DIR/tools/"
fi

# Include Ruby binding sources since the workspace now depends on packages/ruby/.
mkdir -p "$WORKSPACE_DIR/packages/ruby"
rsync -a --exclude 'target' --exclude 'native/target' "$ROOT/packages/ruby/" "$WORKSPACE_DIR/packages/ruby/"

# Mirror workspace for packages/php-ext to satisfy configure scripts that resolve relative paths.
WORKSPACE_ALT_DIR="$STAGING/packages/php-ext/workspace"
mkdir -p "$WORKSPACE_ALT_DIR"
rsync -a --exclude 'target' --exclude 'debug' "$WORKSPACE_DIR/" "$WORKSPACE_ALT_DIR/"

# PIE metadata required by the CLI
cp "$ROOT/composer.json" "$STAGING/"
cp "$EXT_SRC_DIR/README.md" "$STAGING/PIE-README.md"

ARCHIVE_NAME="php_html_to_markdown-${VERSION}-src.tgz"
tar -czf "${DEST_DIR}/${ARCHIVE_NAME}" -C "$STAGING" .

echo "Created ${DEST_DIR}/${ARCHIVE_NAME}"
