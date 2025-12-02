#!/usr/bin/env bash
set -euo pipefail

pushd packages/elixir >/dev/null
env MIX_ENV=test mix test
popd >/dev/null
