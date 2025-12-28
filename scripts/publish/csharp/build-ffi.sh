#!/usr/bin/env bash
set -euo pipefail

rid="${1:?RID is required (e.g. linux-x64, osx-arm64, win-x64)}"
out_dir="${2:?OUT_DIR is required}"

mkdir -p "${out_dir}/${rid}/native"

cargo build --release -p html-to-markdown-ffi

case "${rid}" in
win-*)
	lib_path="$(find target/release -maxdepth 1 -type f -name '*html_to_markdown_ffi*.dll' -print -quit)"
	;;
osx-*)
	lib_path="$(find target/release -maxdepth 1 -type f -name 'libhtml_to_markdown_ffi*.dylib' -print -quit)"
	;;
linux-*)
	lib_path="$(find target/release -maxdepth 1 -type f -name 'libhtml_to_markdown_ffi*.so' -print -quit)"
	;;
*)
	echo "Unsupported RID: ${rid}" >&2
	exit 1
	;;
esac

if [[ -z "${lib_path:-}" || ! -f "${lib_path}" ]]; then
	echo "Failed to locate built html_to_markdown_ffi library for ${rid}" >&2
	echo "Contents of target/release:" >&2
	ls -la target/release/*.{dll,dylib,so} 2>/dev/null || ls -la target/release/ || true
	exit 1
fi

cp -f "${lib_path}" "${out_dir}/${rid}/native/"
