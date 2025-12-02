#!/usr/bin/env bash
set -euo pipefail

pushd packages/ruby >/dev/null
bundle exec rake clean
ruby ../../scripts/prepare_ruby_gem.rb
bundle exec rake build
popd >/dev/null
