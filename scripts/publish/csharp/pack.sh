#!/usr/bin/env bash
set -euo pipefail

if [[ -d "dist/csharp-ffi" ]]; then
	scripts/publish/csharp/stage-ffi.sh "dist/csharp-ffi" "packages/csharp/HtmlToMarkdown"
fi

dotnet pack packages/csharp/HtmlToMarkdown/HtmlToMarkdown.csproj --configuration Release --output artifacts/csharp
