#!/usr/bin/env bash
set -euo pipefail

gem_dir="${1:-ruby-gems}"

: "${GITHUB_OUTPUT:?GITHUB_OUTPUT is required}"

if [[ ! -d "${gem_dir}" ]]; then
	echo "error: gem directory not found: ${gem_dir}" >&2
	exit 1
fi

gem_file="$(
	find "${gem_dir}" -maxdepth 1 -type f -name '*.gem' -print |
		LC_ALL=C sort |
		head -n 1
)"

: "${gem_file:?No .gem found in ${gem_dir}}"

echo "path=${gem_file}" >>"${GITHUB_OUTPUT}"
