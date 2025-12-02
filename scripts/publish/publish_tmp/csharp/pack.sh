#!/usr/bin/env bash
set -euo pipefail

dotnet pack packages/csharp/HtmlToMarkdown/HtmlToMarkdown.csproj --configuration Release --output artifacts/csharp
