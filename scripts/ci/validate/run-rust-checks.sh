#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo clippy --workspace --exclude html-to-markdown-rb --exclude html-to-markdown-php -- -D warnings
