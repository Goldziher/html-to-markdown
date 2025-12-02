#!/usr/bin/env bash
set -euo pipefail

pushd packages/elixir >/dev/null
mix hex.publish --yes
popd >/dev/null
