#!/usr/bin/env bash
set -euo pipefail

tag="${TAG:?TAG is required}"

shopt -s nullglob
for archive in dist/go-ffi/*/*.{tar.gz,zip}; do
	if [ -f "$archive" ]; then
		gh release upload "${tag}" "${archive}" --clobber
	fi
done
