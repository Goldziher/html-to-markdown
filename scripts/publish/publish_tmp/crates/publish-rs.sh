#!/usr/bin/env bash
set -euo pipefail

cargo publish -p html-to-markdown-rs --token "${CARGO_TOKEN:?CARGO_TOKEN is required}"
