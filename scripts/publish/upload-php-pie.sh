#!/usr/bin/env bash
set -euo pipefail

tag="${TAG:?TAG is required}"
version="${VERSION:?VERSION is required}"

gh release upload "${tag}" "dist/php-pie/php_html_to_markdown-${version}-src.tgz" --clobber
