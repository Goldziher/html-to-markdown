#!/usr/bin/env bash
set -euo pipefail

pushd crates/html-to-markdown-node >/dev/null
pnpm test
popd >/dev/null
