#!/usr/bin/env bash
set -euo pipefail

env MIX_ENV=dev mix credo --strict
