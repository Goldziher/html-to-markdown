#!/usr/bin/env bash
set -euo pipefail

# Ensure RI_DEVKIT is set; default to common RubyInstaller devkit roots on GH runners.
if [[ -z "${RI_DEVKIT:-}" ]]; then
  if [[ -d "/ucrt64" ]]; then
    RI_DEVKIT="/ucrt64"
  elif [[ -d "C:/msys64" ]]; then
    RI_DEVKIT="C:/msys64"
  else
    echo "RI_DEVKIT is unset and no default devkit path found" >&2
    exit 1
  fi
fi

# Normalize Windows-style path to POSIX for clang.
RI_DEVKIT_POSIX="${RI_DEVKIT//\\/\/}"
MSYSTEM_PREFIX="${MSYSTEM_PREFIX:-/ucrt64}"
echo "BINDGEN_EXTRA_CLANG_ARGS=--target=x86_64-pc-windows-gnu --sysroot=${RI_DEVKIT_POSIX}${MSYSTEM_PREFIX}" >> "$GITHUB_ENV"
