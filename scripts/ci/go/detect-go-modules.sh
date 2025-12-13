#!/usr/bin/env bash
set -euo pipefail

mapfile -t modules < <(git ls-files -- '*/go.mod' | sort)
if [[ "${#modules[@]}" -eq 0 ]]; then
	echo "modules=[]" >>"$GITHUB_OUTPUT"
	exit 0
fi

json=$(printf '%s\n' "${modules[@]}" | sed 's|/go\.mod$||' | jq -R -s -c 'split("\n") | map(select(length > 0))')
echo "modules=$json" >>"$GITHUB_OUTPUT"
