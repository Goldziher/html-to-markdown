#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

SRC_DIR="${ROOT_DIR}/crates/html-to-markdown"
DEST_DIR="${ROOT_DIR}/packages/elixir/native/html_to_markdown_elixir/vendor/html-to-markdown-rs"

if [[ ! -d "${SRC_DIR}" ]]; then
	echo "Missing Rust core crate at ${SRC_DIR}" >&2
	exit 1
fi

rm -rf "${DEST_DIR}"
mkdir -p "$(dirname "${DEST_DIR}")"

if command -v rsync >/dev/null 2>&1; then
	rsync -a --delete --exclude target --exclude .git "${SRC_DIR}/" "${DEST_DIR}/"
else
	cp -R "${SRC_DIR}" "${DEST_DIR}"
	rm -rf "${DEST_DIR}/target" "${DEST_DIR}/.git" || true
fi
