#!/usr/bin/env bash
set -euo pipefail

out_dir="wasm-artifacts"
rm -rf "${out_dir}"
mkdir -p "${out_dir}"
for folder in dist dist-node dist-web; do
	if [ -d "crates/html-to-markdown-wasm/${folder}" ]; then
		tar -czf "${out_dir}/html-to-markdown-${folder}.tar.gz" -C crates/html-to-markdown-wasm "${folder}"
	fi
done
