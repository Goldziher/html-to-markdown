#!/usr/bin/env bash
set -euo pipefail

platform="${1:?platform is required (linux-x64, linux-arm64, darwin-x64, darwin-arm64, windows-x64)}"
version="${2:?version is required}"
out_dir="${3:?out_dir is required}"

case "${platform}" in
linux-x64)
	rid="linux-x64"
	archive_ext="tar.gz"
	target_name="libhtml_to_markdown_ffi.so"
	;;
linux-arm64)
	rid="linux-arm64"
	archive_ext="tar.gz"
	target_name="libhtml_to_markdown_ffi.so"
	;;
darwin-x64)
	rid="osx-x64"
	archive_ext="tar.gz"
	target_name="libhtml_to_markdown_ffi.dylib"
	;;
darwin-arm64)
	rid="osx-arm64"
	archive_ext="tar.gz"
	target_name="libhtml_to_markdown_ffi.dylib"
	;;
windows-x64)
	rid="win-x64"
	archive_ext="zip"
	target_name="html_to_markdown_ffi.dll"
	;;
*)
	echo "Unsupported platform: ${platform}" >&2
	exit 1
	;;
esac

work_dir="$(mktemp -d)"
trap 'rm -rf "${work_dir}"' EXIT

scripts/publish/csharp/build-ffi.sh "${rid}" "${work_dir}"

lib_path="$(find "${work_dir}/${rid}/native" -maxdepth 1 -type f -name '*html_to_markdown_ffi*' -print -quit)"
if [[ -z "${lib_path}" ]]; then
	echo "Failed to locate html_to_markdown_ffi library for ${platform}" >&2
	exit 1
fi

mkdir -p "${out_dir}"

archive_name="html-to-markdown-ffi-${version}-${platform}.${archive_ext}"
stage_dir="${work_dir}/stage"
mkdir -p "${stage_dir}"
cp -f "${lib_path}" "${stage_dir}/${target_name}"

if [[ "${archive_ext}" == "zip" ]]; then
	if command -v zip >/dev/null 2>&1; then
		(
			cd "${stage_dir}"
			zip -9 -q "${out_dir}/${archive_name}" "${target_name}"
		)
	else
		powershell.exe -NoProfile -Command \
			"Compress-Archive -Path \"${stage_dir}\\${target_name}\" -DestinationPath \"${out_dir}\\${archive_name}\""
	fi
else
	tar -czf "${out_dir}/${archive_name}" -C "${stage_dir}" "${target_name}"
fi

echo "Packaged ${out_dir}/${archive_name}"
