#!/usr/bin/env bash
set -euo pipefail

: "${VERSION:?VERSION is required}"

exists=$(
	python3 - <<'PY'
import json, os, urllib.request
version = os.environ["VERSION"]
with urllib.request.urlopen("https://hex.pm/api/packages/html_to_markdown") as resp:
    data = json.load(resp)
exists = any(rel.get("version") == version for rel in data.get("releases", []))
print("true" if exists else "false")
PY
)

echo "exists=${exists}" >>"${GITHUB_OUTPUT}"
