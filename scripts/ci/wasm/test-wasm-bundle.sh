#!/usr/bin/env bash
set -euo pipefail

VITEST_TIMEOUT_MS="${VITEST_TIMEOUT_MS:-60000}"

pnpm vitest run \
	--reporter=verbose \
	--pool=threads \
	--maxWorkers=1 \
	--exclude=".venv/**" \
	--test-timeout="${VITEST_TIMEOUT_MS}"
