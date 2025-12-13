#!/usr/bin/env bash
set -euo pipefail

: "${RUNNER_TEMP:?RUNNER_TEMP is required}"
: "${GITHUB_PATH:?GITHUB_PATH is required}"

maven_version="$(
	curl -fsSL "https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/maven-metadata.xml" |
		sed -n 's:.*<release>\(.*\)</release>.*:\1:p' |
		head -n 1
)"
: "${maven_version:?Failed to determine Maven version}"

tmp_dir="${RUNNER_TEMP}"
if [[ "${RUNNER_OS:-}" == "Windows" ]]; then
	if command -v cygpath >/dev/null 2>&1; then
		tmp_dir="$(cygpath -u "${RUNNER_TEMP}")"
	fi
fi

maven_dir="${tmp_dir}/apache-maven-${maven_version}"
archive_path="${tmp_dir}/maven.tar.gz"

if [[ ! -d "${maven_dir}" ]]; then
	curl -fsSL \
		"https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/${maven_version}/apache-maven-${maven_version}-bin.tar.gz" \
		-o "${archive_path}"
	tar -xzf "${archive_path}" -C "${tmp_dir}"
fi

echo "${maven_dir}/bin" >>"${GITHUB_PATH}"
