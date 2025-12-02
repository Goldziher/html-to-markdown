#!/usr/bin/env bash
set -euo pipefail

golangci-lint run --config ../../.golangci.yml ./...
