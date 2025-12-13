#!/usr/bin/env bash
set -euo pipefail

version="${VERSION:?VERSION is required}"

exists=$(
	python - "$version" <<'PY'
import json, sys, urllib.request
version = sys.argv[1]
try:
    with urllib.request.urlopen("https://pypi.org/pypi/html-to-markdown/json") as resp:
        data = json.load(resp)
    exists = version in data.get("releases", {})
except Exception:
    exists = False
print("true" if exists else "false")
PY
)

echo "exists=${exists}" >>"${GITHUB_OUTPUT}"
