#!/usr/bin/env bash
set -euo pipefail

pnpm --filter html-to-markdown-node exec napi build --release
mkdir -p typescript-defs
cp crates/html-to-markdown-node/index.js crates/html-to-markdown-node/index.d.ts typescript-defs/
