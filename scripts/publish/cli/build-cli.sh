#!/usr/bin/env bash
set -euo pipefail

target="${TARGET:?TARGET is required}"
use_cross="${USE_CROSS:-false}"

if [[ "${use_cross}" == "true" ]]; then
	cross build --release --target "${target}" --package html-to-markdown-cli
else
	cargo build --release --target "${target}" --package html-to-markdown-cli
fi
