#!/usr/bin/env bash
set -euo pipefail

VITEST_TIMEOUT_MS="${VITEST_TIMEOUT_MS:-60000}"

# More verbose output and a generous per-test timeout to help debug hangs in CI.
# Vitest v4 does not support --runInBand; use --threads=false to serialize.
pnpm vitest run --reporter=verbose --threads=false --test-timeout="${VITEST_TIMEOUT_MS}"
