#!/usr/bin/env bash
set -euo pipefail

shopt -s nullglob
files=(packages/ruby/pkg/*.gem)
if [ ${#files[@]} -eq 0 ]; then
	echo "No gems were produced" >&2
	exit 1
fi
