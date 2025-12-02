#!/usr/bin/env bash
set -euo pipefail

echo "Running Rust tests with full debugging..."
cargo test --release --no-default-features --workspace --exclude html-to-markdown-rb --exclude html-to-markdown-php --exclude html-to-markdown-wasm-wasmtime-tests -vv -- --nocapture --test-threads=1
