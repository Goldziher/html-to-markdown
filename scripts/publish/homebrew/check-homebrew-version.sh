#!/usr/bin/env bash
set -euo pipefail

: "${VERSION:?VERSION is required}"

TAP_URL="https://raw.githubusercontent.com/kreuzberg-dev/homebrew-tap/HEAD/Formula/html-to-markdown.rb"
formula_content="$(curl -sSL "${TAP_URL}" || true)"

current_version="$(printf "%s\n" "${formula_content}" | awk -F'"' '/^  version "/ { print $2; exit }')"

exists=false
if [[ -n "${current_version:-}" && "${current_version}" == "${VERSION}" ]]; then
	exists=true
fi

echo "exists=${exists}" >>"${GITHUB_OUTPUT}"
