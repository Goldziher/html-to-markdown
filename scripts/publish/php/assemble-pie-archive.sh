#!/usr/bin/env bash
set -euo pipefail

version="${VERSION:?VERSION is required}"
./scripts/package_php_pie_source.sh "${version}" build/artifacts
