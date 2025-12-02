#!/usr/bin/env bash
set -euo pipefail

target="${TARGET:?TARGET is required}"

pnpm --filter html-to-markdown-node exec napi artifacts --output-dir ./artifacts
test -d crates/html-to-markdown-node/npm || { echo "npm artifact directory missing"; exit 1; }
tar -czf "node-bindings-${target}.tar.gz" -C crates/html-to-markdown-node npm
