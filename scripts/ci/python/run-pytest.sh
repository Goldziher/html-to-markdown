#!/usr/bin/env bash
set -euo pipefail

PYTEST_ADDOPTS="${PYTEST_ADDOPTS:--vv --maxfail=1 --durations=25}"

IFS=" " read -r -a pytest_addopts <<<"$PYTEST_ADDOPTS"
uv pip install --editable packages/python
uv run pytest "${pytest_addopts[@]}"
