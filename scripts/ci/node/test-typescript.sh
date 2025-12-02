#!/usr/bin/env bash
set -euo pipefail

pushd packages/typescript >/dev/null
pnpm test
popd >/dev/null
