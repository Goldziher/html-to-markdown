#!/usr/bin/env bash
set -euo pipefail

echo "Dry run requested; gem artifacts ready:" >> "${GITHUB_STEP_SUMMARY}"
ls -1 html-to-markdown-*.gem >> "${GITHUB_STEP_SUMMARY}"
