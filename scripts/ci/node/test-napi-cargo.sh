#!/usr/bin/env bash
set -euo pipefail

pushd crates/html-to-markdown-node >/dev/null
cargo test
popd >/dev/null
