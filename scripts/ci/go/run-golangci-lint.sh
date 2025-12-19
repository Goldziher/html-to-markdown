#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"

golangci-lint run --config "${ROOT_DIR}/.golangci.yml" ./...
