#!/usr/bin/env bash
set -euo pipefail

version="${VERSION:?VERSION is required}"
pkg="goldziher.htmltomarkdown"

exists=$(
	python3 - "$pkg" "$version" <<'PY'
import json, sys, urllib.request
pkg = sys.argv[1]
version = sys.argv[2]
try:
    with urllib.request.urlopen(f"https://api.nuget.org/v3-flatcontainer/{pkg}/index.json") as resp:
        data = json.load(resp)
    exists = version in data.get("versions", [])
except Exception:
    exists = False
print("true" if exists else "false")
PY
)

echo "exists=${exists}" >>"${GITHUB_OUTPUT}"
