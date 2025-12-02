#!/usr/bin/env bash
set -euo pipefail

cargo llvm-cov --workspace --exclude html-to-markdown-py --exclude html-to-markdown-rb --exclude html-to-markdown-php --exclude html-to-markdown-node --exclude html-to-markdown-wasm --exclude html-to-markdown-wasm-wasmtime-tests --all-features --lcov --output-path coverage.lcov
