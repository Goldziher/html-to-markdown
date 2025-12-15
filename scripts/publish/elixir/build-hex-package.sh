#!/usr/bin/env bash
set -euo pipefail

scripts/publish/elixir/stage-rust-core.sh

pushd packages/elixir >/dev/null
mix hex.build
popd >/dev/null
