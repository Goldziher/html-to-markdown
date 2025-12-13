#!/usr/bin/env bash
set -euo pipefail

if [[ -n "${EXTENSION_PATH:-}" ]]; then
	ini_file="$(mktemp)"
	echo "extension=${EXTENSION_PATH}" >"${ini_file}"
	export PHPRC="${ini_file}"
fi

pushd packages/php >/dev/null
composer run test
popd >/dev/null
