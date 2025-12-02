#!/usr/bin/env bash
set -euo pipefail

os="${MATRIX_OS:?MATRIX_OS is required}"

case "${os}" in
  ubuntu-latest)
    echo "label=linux" >> "${GITHUB_OUTPUT}"
    ;;
  macos-latest)
    echo "label=macos-arm64" >> "${GITHUB_OUTPUT}"
    ;;
  windows-latest)
    echo "label=windows-x64" >> "${GITHUB_OUTPUT}"
    ;;
  *)
    echo "Unsupported matrix.os=${os}" >&2
    exit 1
    ;;
esac
