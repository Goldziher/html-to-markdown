#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

SRC_DIR="${ROOT_DIR}/crates/html-to-markdown"
DEST_DIR="${ROOT_DIR}/packages/elixir/native/html_to_markdown_elixir/vendor/html-to-markdown-rs"

VERSION="$(
	python3 - "${ROOT_DIR}/Cargo.toml" <<'PY'
import re
from pathlib import Path
import sys

text = Path(sys.argv[1]).read_text(encoding="utf-8")
in_workspace_pkg = False
for line in text.splitlines():
    if line.strip() == "[workspace.package]":
        in_workspace_pkg = True
        continue
    if in_workspace_pkg and line.startswith("[") and line.strip().startswith("[") and line.strip() != "[workspace.package]":
        in_workspace_pkg = False
    if in_workspace_pkg:
        m = re.match(r'version\s*=\s*"([^"]+)"\s*$', line.strip())
        if m:
            print(m.group(1))
            raise SystemExit(0)
raise SystemExit("Failed to find [workspace.package] version in Cargo.toml")
PY
)"

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

python3 - "${DEST_DIR}/Cargo.toml" "${VERSION}" <<'PY'
import re
import sys
from pathlib import Path

path = Path(sys.argv[1])
version = sys.argv[2]
text = path.read_text(encoding="utf-8")

replacements = {
    r"^version\.workspace\s*=\s*true\s*$": f'version = "{version}"',
    r"^edition\.workspace\s*=\s*true\s*$": 'edition = "2024"',
    r"^authors\.workspace\s*=\s*true\s*$": 'authors = ["Na\'aman Hirschfeld <nhirschfeld@gmail.com>"]',
    r"^license\.workspace\s*=\s*true\s*$": 'license = "MIT"',
    r"^repository\.workspace\s*=\s*true\s*$": 'repository = "https://github.com/kreuzberg-dev/html-to-markdown"',
    r"^homepage\.workspace\s*=\s*true\s*$": 'homepage = "https://github.com/kreuzberg-dev/html-to-markdown"',
    r"^documentation\.workspace\s*=\s*true\s*$": 'documentation = "https://docs.rs/html-to-markdown-rs"',
    r"^rust-version\.workspace\s*=\s*true\s*$": 'rust-version = "1.85"',
    r"^\[lints\]\s*\nworkspace\s*=\s*true\s*$": '[lints]\nrust.unsafe_code = "forbid"\nrust.missing_docs = "warn"\nrust.unused_must_use = "deny"',
    r"^tl\.workspace\s*=\s*true\s*$": 'tl = { package = "astral-tl", version = "0.7.11" }',
    r"^regex\.workspace\s*=\s*true\s*$": 'regex = "1.12"',
    r"^once_cell\.workspace\s*=\s*true\s*$": 'once_cell = "1.21"',
    r"^thiserror\.workspace\s*=\s*true\s*$": 'thiserror = "2.0"',
    r"^base64\.workspace\s*=\s*true\s*$": 'base64 = "0.22"',
    r"^html5ever\.workspace\s*=\s*true\s*$": 'html5ever = "0.36"',
    r"^markup5ever_rcdom\.workspace\s*=\s*true\s*$": 'markup5ever_rcdom = "0.36"',
    r"^async-trait\s*=\s*{\s*workspace\s*=\s*true,\s*optional\s*=\s*true\s*}\s*$": 'async-trait = { version = "0.1", optional = true }',
}

for pattern, replacement in replacements.items():
    text = re.sub(pattern, replacement, text, flags=re.MULTILINE)

path.write_text(text, encoding="utf-8")
PY

# Add #![allow(unused)] to all converter module files as inner attribute
# since visitor feature gates many imports that are unused when visitor is disabled
find "${DEST_DIR}/src/converter" -type f -name "*.rs" -print0 | while IFS= read -r -d '' file; do
	# Check if file already has #![allow(unused)]
	if ! grep -q "^\s*#!\[allow(unused)" "$file"; then
		# Add #![allow(unused)] at the very beginning of the file (before doc comments)
		python3 - "$file" <<'PYFIX'
import sys
from pathlib import Path

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")

# Add inner attribute at the very beginning
text = '#![allow(unused)]\n' + text

path.write_text(text, encoding="utf-8")
PYFIX
	fi
done
