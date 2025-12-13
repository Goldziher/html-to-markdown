#!/usr/bin/env bash
set -euo pipefail

tag_version="${TAG_VERSION:?TAG_VERSION is required}"
cargo_version=$(grep '^version = ' Cargo.toml | head -1 | sed -E 's/version = "(.*)"/\1/')

if [[ "${cargo_version}" != "${tag_version}" ]]; then
	echo "Version mismatch! Cargo: ${cargo_version}, tag: ${tag_version}" >&2
	exit 1
fi

echo "Cargo.toml version matches tag: ${cargo_version}"
