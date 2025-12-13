#!/usr/bin/env bash
set -euo pipefail

echo "$HOME/go/bin" >>"$GITHUB_PATH"
go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@"${GOLANGCI_LINT_VERSION}"
