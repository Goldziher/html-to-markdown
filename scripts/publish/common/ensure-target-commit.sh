#!/usr/bin/env bash
set -euo pipefail

target_sha="${TARGET_SHA:-}"

if [[ -z "${target_sha}" ]]; then
  exit 0
fi

git checkout --progress --force "${target_sha}"
