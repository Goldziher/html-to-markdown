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

rsync -a "$EXT_SRC_DIR"/ "$STAGING"/
mkdir -p "$STAGING/packages/php-ext"
rsync -a "$EXT_SRC_DIR"/ "$STAGING/packages/php-ext"/

WORKSPACE_DIR="$STAGING/workspace"
mkdir -p "$WORKSPACE_DIR"

cp "$ROOT/Cargo.toml" "$WORKSPACE_DIR/"
if [[ -f "$ROOT/Cargo.lock" ]]; then
	cp "$ROOT/Cargo.lock" "$WORKSPACE_DIR/"
fi
cp "$ROOT/LICENSE" "$STAGING/"
cp "$ROOT/README.md" "$STAGING/PROJECT-README.md"

WORKSPACE_CARGO="$WORKSPACE_DIR/Cargo.toml"
python3 - "$WORKSPACE_CARGO" <<'PY'
from pathlib import Path
import sys

cargo_path = Path(sys.argv[1])
lines = [
    line for line in cargo_path.read_text().splitlines()
    if '"e2e/wasm-wasmtime"' not in line
]
cargo_path.write_text("\n".join(lines) + "\n")
PY

rsync -a --exclude 'target' --exclude 'debug' "$ROOT/crates" "$WORKSPACE_DIR/"

if [[ -d "$ROOT/tools" ]]; then
	mkdir -p "$WORKSPACE_DIR/tools"
	rsync -a --exclude 'target' --exclude 'debug' "$ROOT/tools/" "$WORKSPACE_DIR/tools/"
fi

mkdir -p "$WORKSPACE_DIR/packages/ruby"
rsync -a --exclude 'target' --exclude 'native/target' "$ROOT/packages/ruby/" "$WORKSPACE_DIR/packages/ruby/"

mkdir -p "$WORKSPACE_DIR/packages/elixir"
rsync -a --exclude '_build' --exclude 'deps' --exclude 'native/html_to_markdown_elixir/target' \
	"$ROOT/packages/elixir/" "$WORKSPACE_DIR/packages/elixir/"

WORKSPACE_ALT_DIR="$STAGING/packages/php-ext/workspace"
mkdir -p "$WORKSPACE_ALT_DIR"
rsync -a --exclude 'target' --exclude 'debug' "$WORKSPACE_DIR/" "$WORKSPACE_ALT_DIR/"

cp "$ROOT/composer.json" "$STAGING/"
cp "$EXT_SRC_DIR/README.md" "$STAGING/PIE-README.md"

ARCHIVE_NAME="php_html_to_markdown-${VERSION}-src.tgz"
tar -czf "${DEST_DIR}/${ARCHIVE_NAME}" -C "$STAGING" .

echo "Created ${DEST_DIR}/${ARCHIVE_NAME}"
