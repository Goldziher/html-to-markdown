#!/usr/bin/env bash
set -euo pipefail

src_dir="${1:?SRC_DIR is required}"
project_dir="${2:-packages/csharp/HtmlToMarkdown}"

rm -rf "${project_dir}/runtimes"
mkdir -p "${project_dir}/runtimes"

for rid in "${src_dir}"/*; do
	[[ -d "${rid}" ]] || continue
	rid_name="$(basename "${rid}")"
	mkdir -p "${project_dir}/runtimes/${rid_name}/native"
	cp -f "${rid}/native/"* "${project_dir}/runtimes/${rid_name}/native/"
done
