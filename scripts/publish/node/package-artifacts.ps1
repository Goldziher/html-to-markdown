$ErrorActionPreference = "Stop"

$target = $env:TARGET
if (-not $target) { throw "TARGET is required" }
pnpm --filter html-to-markdown-node exec napi artifacts --output-dir ./artifacts
if (-Not (Test-Path crates/html-to-markdown-node\npm)) { throw "npm artifact directory missing" }
tar -czf "node-bindings-$target.tar.gz" -C crates/html-to-markdown-node npm
