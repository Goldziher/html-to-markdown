#!/usr/bin/env bash
set -euo pipefail

rustup target add wasm32-unknown-unknown
rustc --print target-libdir --target wasm32-unknown-unknown
