#!/usr/bin/env bash
set -euo pipefail

scripts/publish/elixir/stage-rust-core.sh

env MIX_ENV=test mix test
