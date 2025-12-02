#!/usr/bin/env bash
set -euo pipefail

rm -rf crates/html-to-markdown-node/npm
mkdir -p crates/html-to-markdown-node
for pkg in node-artifacts/*.tar.gz; do
  tar -xzf "${pkg}" -C crates/html-to-markdown-node
done
cp typescript-defs/index.js typescript-defs/index.d.ts crates/html-to-markdown-node/
