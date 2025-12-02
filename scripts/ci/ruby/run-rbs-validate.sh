#!/usr/bin/env bash
set -euo pipefail

pushd packages/ruby >/dev/null
bundle exec rbs validate
popd >/dev/null
