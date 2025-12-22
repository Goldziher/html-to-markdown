#!/usr/bin/env bash
set -euo pipefail

binary_name="html-to-markdown"
if [[ "${RUNNER_OS:-}" == "Windows" ]]; then
	binary_name="html-to-markdown.exe"
fi
rm -f "target/release/${binary_name}"
cargo build --release --package html-to-markdown-cli
