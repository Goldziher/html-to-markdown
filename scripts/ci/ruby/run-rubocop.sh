#!/usr/bin/env bash
set -euo pipefail

pushd packages/ruby >/dev/null
bundle exec rubocop --config .rubocop.yml
popd >/dev/null
