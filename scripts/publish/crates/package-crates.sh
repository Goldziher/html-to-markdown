#!/usr/bin/env bash
set -euo pipefail

release_version="${RELEASE_VERSION:-unknown}"

cargo package -p html-to-markdown-rs --allow-dirty

cli_packaged=0
cli_status=0
cargo package -p html-to-markdown-cli --allow-dirty --no-verify || cli_status=$?

if [[ "${cli_status}" -eq 0 ]]; then
  cli_packaged=1
else
  echo "::warning::Skipping html-to-markdown-cli crate packaging; html-to-markdown-rs ${release_version} is not yet available on crates.io."
  if [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
    {
      echo "### html-to-markdown-cli crate"
      echo ""
      echo "- Packaging skipped because html-to-markdown-rs ${release_version} is not yet published to crates.io."
    } >> "${GITHUB_STEP_SUMMARY}"
  fi
fi

mkdir -p crate-artifacts
cp target/package/html-to-markdown-rs-*.crate crate-artifacts/
if [[ "${cli_packaged}" -eq 1 ]]; then
  cp target/package/html-to-markdown-cli-*.crate crate-artifacts/
fi
