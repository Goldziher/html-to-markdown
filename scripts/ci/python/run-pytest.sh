#!/usr/bin/env bash
set -euo pipefail

PYTEST_ADDOPTS="${PYTEST_ADDOPTS:--vv --maxfail=1 --durations=25}"

uv run pytest ${PYTEST_ADDOPTS}
