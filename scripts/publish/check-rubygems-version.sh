#!/usr/bin/env bash
set -euo pipefail

: "${VERSION:?VERSION is required}"

exists=$(python3 - <<'PY'
import json, os, urllib.request
version = os.environ["VERSION"]
with urllib.request.urlopen("https://rubygems.org/api/v1/versions/html-to-markdown.json") as resp:
    data = json.load(resp)
exists = any(entry.get("number") == version for entry in data)
print("true" if exists else "false")
PY
)

echo "exists=${exists}" >> "${GITHUB_OUTPUT}"
