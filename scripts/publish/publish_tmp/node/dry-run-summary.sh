#!/usr/bin/env bash
set -euo pipefail

echo "Dry run requested; Node binding tarballs staged:" >> "${GITHUB_STEP_SUMMARY}"
find node-artifacts -name '*.tar.gz' -printf '%f\n' >> "${GITHUB_STEP_SUMMARY}"
