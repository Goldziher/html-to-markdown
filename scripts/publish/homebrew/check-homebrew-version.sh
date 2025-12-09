#!/usr/bin/env bash
set -euo pipefail

: "${VERSION:?VERSION is required}"

TAP_URL="https://raw.githubusercontent.com/Goldziher/homebrew-tap/HEAD/Formula/html-to-markdown.rb"
current_version="$(curl -fsSL "${TAP_URL}" | awk -F'"' '/^  version "/ { print $2; exit }')"

exists=false
if [[ -n "${current_version:-}" && "${current_version}" == "${VERSION}" ]]; then
  exists=true
fi

echo "exists=${exists}" >> "${GITHUB_OUTPUT}"
