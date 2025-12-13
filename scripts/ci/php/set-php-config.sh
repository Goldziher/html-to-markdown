#!/usr/bin/env bash
set -euo pipefail

echo "PHP_CONFIG=$(command -v php-config)" >>"$GITHUB_ENV"
