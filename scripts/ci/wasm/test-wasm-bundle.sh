#!/usr/bin/env bash
set -euo pipefail

VITEST_TIMEOUT_MS="${VITEST_TIMEOUT_MS:-60000}"

# More verbose output and a generous per-test timeout to help debug hangs in CI.
pnpm vitest run --reporter=verbose --runInBand --test-timeout="${VITEST_TIMEOUT_MS}"
