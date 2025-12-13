#!/usr/bin/env bash
set -euo pipefail

cd crates/html-to-markdown-node/npm
for dir in */; do
	if [ -f "$dir/package.json" ]; then
		(cd "$dir" && npm pack && mv ./*.tgz ..)
	fi
done
