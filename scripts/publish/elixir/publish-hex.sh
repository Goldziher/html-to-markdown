#!/usr/bin/env bash
set -euo pipefail

scripts/publish/elixir/stage-rust-core.sh

pushd packages/elixir >/dev/null
mix hex.publish --yes
popd >/dev/null
