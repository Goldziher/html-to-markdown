#!/usr/bin/env bash
set -euo pipefail

echo "BINDGEN_EXTRA_CLANG_ARGS=--target=x86_64-pc-windows-gnu --sysroot=${RI_DEVKIT//\\/\/}$MSYSTEM_PREFIX" >> "$GITHUB_ENV"
