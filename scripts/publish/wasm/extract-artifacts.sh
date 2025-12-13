#!/usr/bin/env bash
set -euo pipefail

cd wasm-artifacts
for tarball in *.tar.gz; do
	tar -xzf "${tarball}" -C ../crates/html-to-markdown-wasm
done
