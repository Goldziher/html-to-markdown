#!/usr/bin/env bash
set -euo pipefail

VITEST_TIMEOUT_MS="${VITEST_TIMEOUT_MS:-60000}"

# More verbose output and a generous per-test timeout to help debug hangs in CI.
# Vitest v4 removed the old --threads flag; cap workers explicitly instead.
pnpm vitest run \
  --reporter=verbose \
  --pool=threads \
  --maxWorkers=1 \
  --exclude=".venv/**" \
  --test-timeout="${VITEST_TIMEOUT_MS}"
