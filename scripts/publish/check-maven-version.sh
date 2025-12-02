#!/usr/bin/env bash
set -euo pipefail

version="${VERSION:?VERSION is required}"
base_url="https://repo1.maven.org/maven2/io/github/goldziher/html-to-markdown/${version}"
status_code=$(curl -s -o /dev/null -w "%{http_code}" "${base_url}/html-to-markdown-${version}.pom")

if [[ "${status_code}" == "200" ]]; then
  echo "exists=true" >> "${GITHUB_OUTPUT}"
else
  echo "exists=false" >> "${GITHUB_OUTPUT}"
fi
