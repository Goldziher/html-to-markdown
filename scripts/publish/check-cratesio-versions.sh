#!/usr/bin/env bash
set -euo pipefail

version="${VERSION:?VERSION is required}"

check_crate() {
	crate="$1"
	python - "$crate" "$version" <<'PY'
import json, sys, urllib.request
crate = sys.argv[1]
version = sys.argv[2]
try:
    with urllib.request.urlopen(f"https://crates.io/api/v1/crates/{crate}") as resp:
        data = json.load(resp)
    versions = [item.get("num") for item in data.get("versions", [])]
    print("true" if version in versions else "false")
except Exception:
    print("false")
PY
}

rs_exists=$(check_crate "html-to-markdown-rs")
cli_exists=$(check_crate "html-to-markdown-cli")

if [[ "${rs_exists}" == "true" && "${cli_exists}" == "true" ]]; then
	all_exist=true
else
	all_exist=false
fi

{
	echo "rs_exists=${rs_exists}"
	echo "cli_exists=${cli_exists}"
	echo "all_exist=${all_exist}"
} >>"${GITHUB_OUTPUT}"
