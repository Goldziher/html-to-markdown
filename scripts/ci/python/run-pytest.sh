#!/usr/bin/env bash
set -euo pipefail

PYTEST_ADDOPTS="${PYTEST_ADDOPTS:--vv --maxfail=1 --durations=25}"

IFS=" " read -r -a pytest_addopts <<<"$PYTEST_ADDOPTS"
binary_name="html-to-markdown"
if [[ "${RUNNER_OS:-}" == "Windows" ]]; then
	binary_name="html-to-markdown.exe"
fi
export HTML_TO_MARKDOWN_CLI="${PWD}/target/release/${binary_name}"
uv pip install --editable packages/python
uv run pytest "${pytest_addopts[@]}"
