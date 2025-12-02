#!/usr/bin/env bash
set -euo pipefail

echo "Dry run requested; artifacts staged for PyPI:" >> "${GITHUB_STEP_SUMMARY}"
ls -1 dist >> "${GITHUB_STEP_SUMMARY}"
