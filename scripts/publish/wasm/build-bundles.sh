#!/usr/bin/env bash
set -euo pipefail

pnpm --filter html-to-markdown-wasm run build:all
