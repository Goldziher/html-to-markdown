#!/usr/bin/env bash
set -euo pipefail

export SKIP=${SKIP:-golangci-lint-packages,golangci-lint-examples}
prek run --show-diff-on-failure --color=always --all-files
