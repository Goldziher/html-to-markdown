#!/usr/bin/env bash
set -euo pipefail

if command -v wasm-pack >/dev/null 2>&1; then
  exit 0
fi

if command -v cargo >/dev/null 2>&1; then
  cargo install wasm-pack --locked
else
  curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi
