#!/usr/bin/env bash
set -euo pipefail

version="${VERSION:?VERSION is required}"
tmp_dir="$(mktemp -d)"
tar -xzf "build/artifacts/php_html_to_markdown-${version}-src.tgz" -C "${tmp_dir}"
echo "Extracted archive contents:"
ls -R "${tmp_dir}"
php build/pie.phar repository:add path "${tmp_dir}"
CARGO_BIN="$(command -v cargo)"
php build/pie.phar build goldziher/html-to-markdown:*@dev --working-dir "${tmp_dir}" --with-cargo-bin="${CARGO_BIN}"
