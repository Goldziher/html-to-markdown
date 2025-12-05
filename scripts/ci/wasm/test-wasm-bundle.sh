#!/usr/bin/env bash
set -euo pipefail

VITEST_TIMEOUT_MS="${VITEST_TIMEOUT_MS:-60000}"

# More verbose output and a generous per-test timeout to help debug hangs in CI.
# Vitest v4 dropped --threads; use the pool single-thread flag instead.
pnpm vitest run \
  --reporter=verbose \
  --pool=threads \
  --poolOptions.threads.singleThread \
  --test-timeout="${VITEST_TIMEOUT_MS}"
