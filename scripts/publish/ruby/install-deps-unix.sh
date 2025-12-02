#!/usr/bin/env bash
set -euo pipefail

pushd packages/ruby >/dev/null
bundle install --jobs 4 --retry 3
popd >/dev/null
