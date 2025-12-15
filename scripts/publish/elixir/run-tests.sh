#!/usr/bin/env bash
set -euo pipefail

scripts/publish/elixir/stage-rust-core.sh

pushd packages/elixir >/dev/null
env MIX_ENV=test mix test
popd >/dev/null
