#!/usr/bin/env bash
set -euo pipefail

PROJECT_PATH="${1:-packages/csharp/HtmlToMarkdown/HtmlToMarkdown.csproj}"

dotnet restore "$PROJECT_PATH"
