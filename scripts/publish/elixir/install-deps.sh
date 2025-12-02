#!/usr/bin/env bash
set -euo pipefail

pushd packages/elixir >/dev/null
mix deps.get
popd >/dev/null
