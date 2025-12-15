#!/usr/bin/env bash
set -euo pipefail

shopt -s nullglob
mapfile -t gems < <(find . -type f -name 'html-to-markdown-*.gem' -print | sort)
if [ "${#gems[@]}" -eq 0 ]; then
	echo "No gem artifacts found" >&2
	exit 1
fi

declare -A pushed_keys=()

for gem in "${gems[@]}"; do
	gem specification "${gem}" >/dev/null

	key="$(
		ruby -r rubygems/package -e '
spec = Gem::Package.new(ARGV[0]).spec
puts [spec.name, spec.version.to_s, spec.platform.to_s].join("|")
' "${gem}"
	)"

	if [[ -n "${pushed_keys[${key}]+x}" ]]; then
		echo "Skipping duplicate gem (${key}): ${gem}"
		continue
	fi

	echo "Pushing ${gem}"
	set +e
	output="$(gem push "${gem}" 2>&1)"
	exit_code=$?
	set -e

	if [[ ${exit_code} -ne 0 ]]; then
		if echo "${output}" | grep -q "Repushing of gem versions is not allowed"; then
			echo "Skipping already-published gem (${key}): ${gem}"
			pushed_keys["${key}"]=1
			continue
		fi
		echo "${output}" >&2
		exit "${exit_code}"
	fi

	echo "${output}"
	pushed_keys["${key}"]=1
done
