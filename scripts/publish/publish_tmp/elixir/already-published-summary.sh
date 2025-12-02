#!/usr/bin/env bash
set -euo pipefail

version="${VERSION:?VERSION is required}"
echo "Hex package ${version} already published; skipping." >> "${GITHUB_STEP_SUMMARY}"
