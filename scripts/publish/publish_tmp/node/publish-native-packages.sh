#!/usr/bin/env bash
set -euo pipefail

shopt -s nullglob
for pkg in crates/html-to-markdown-node/npm/*.tgz; do
  echo "Publishing ${pkg}"
  publish_log=$(mktemp)
  set +e
  npm publish "${pkg}" --access public --ignore-scripts 2>&1 | tee "${publish_log}"
  status=${PIPESTATUS[0]}
  set -e
  if [[ "${status}" -ne 0 ]]; then
    if grep -q "previously published versions" "${publish_log}"; then
      echo "Package ${pkg} already published; skipping."
    else
      exit "${status}"
    fi
  fi
done
