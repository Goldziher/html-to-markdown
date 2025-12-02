#!/usr/bin/env bash
set -euo pipefail

target="${TARGET:?TARGET is required}"
stage="cli-${target}"
rm -rf "${stage}"
mkdir -p "${stage}"
cp "target/${target}/release/html-to-markdown" "${stage}/"
cp LICENSE "${stage}/"
cp README.md "${stage}/"
tar -czf "${stage}.tar.gz" "${stage}"
rm -rf "${stage}"
