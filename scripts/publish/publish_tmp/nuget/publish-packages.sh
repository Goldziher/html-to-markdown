#!/usr/bin/env bash
set -euo pipefail

api_key="${NUGET_API_KEY:?NUGET_API_KEY is required}"

for pkg in dist/nuget/*.nupkg; do
  dotnet nuget push "${pkg}" \
    --api-key "${api_key}" \
    --source https://api.nuget.org/v3/index.json \
    --skip-duplicate
done
