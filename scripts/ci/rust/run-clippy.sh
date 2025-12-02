#!/usr/bin/env bash
set -euo pipefail

cargo clippy -- -D warnings
