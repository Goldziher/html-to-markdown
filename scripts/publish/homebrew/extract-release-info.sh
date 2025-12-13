#!/usr/bin/env bash
set -euo pipefail

tag="${TAG:?TAG is required}"
version="${VERSION:?VERSION is required}"
url="https://github.com/Goldziher/html-to-markdown/archive/${tag}.tar.gz"

{
	echo "tag=${tag}"
	echo "version=${version}"
	echo "url=${url}"
} >>"${GITHUB_OUTPUT}"

{
	echo "Release info:"
	echo "  Tag: ${tag}"
	echo "  Version: ${version}"
	echo "  URL: ${url}"
}
