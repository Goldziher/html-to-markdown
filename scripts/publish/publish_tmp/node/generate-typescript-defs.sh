#!/usr/bin/env bash
set -euo pipefail

# Build without --platform to generate index.d.ts
pnpm --filter html-to-markdown-node exec napi build --release
# Copy index.js (committed) and index.d.ts (generated)
mkdir -p typescript-defs
cp crates/html-to-markdown-node/index.js crates/html-to-markdown-node/index.d.ts typescript-defs/
