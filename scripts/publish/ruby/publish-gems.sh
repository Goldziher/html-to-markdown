#!/usr/bin/env bash
set -euo pipefail

shopt -s nullglob
mapfile -t gems < <(find . -type f -name 'html-to-markdown-*.gem' -print | sort)
if [ "${#gems[@]}" -eq 0 ]; then
	echo "No gem artifacts found" >&2
	exit 1
fi

for gem in "${gems[@]}"; do
	gem specification "${gem}" >/dev/null
	echo "Pushing ${gem}"
	gem push "${gem}"
done
