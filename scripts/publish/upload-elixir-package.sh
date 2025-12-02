#!/usr/bin/env bash
set -euo pipefail

tag="${TAG:?TAG is required}"

shopt -s nullglob
for pkg in dist/elixir/*.tar; do
  gh release upload "${tag}" "${pkg}" --clobber
done
