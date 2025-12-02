#!/usr/bin/env bash
set -euo pipefail

version="${VERSION:?VERSION is required}"
echo "RubyGem version ${version} already published; skipping." >> "${GITHUB_STEP_SUMMARY}"
