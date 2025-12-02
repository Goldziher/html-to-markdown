#!/usr/bin/env bash
set -euo pipefail

pushd packages/php >/dev/null
composer run test
popd >/dev/null
