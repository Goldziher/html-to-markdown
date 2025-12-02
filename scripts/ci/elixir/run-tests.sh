#!/usr/bin/env bash
set -euo pipefail

env MIX_ENV=test mix test
