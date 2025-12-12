#!/usr/bin/env bash
set -euo pipefail

api_key=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --api-key)
      api_key="${2:-}"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ -z "${api_key}" ]]; then
  echo "--api-key is required" >&2
  exit 2
fi

for pkg in dist/nuget/*.nupkg; do
  dotnet nuget push "${pkg}" \
    --api-key "${api_key}" \
    --source https://api.nuget.org/v3/index.json \
    --skip-duplicate
done
