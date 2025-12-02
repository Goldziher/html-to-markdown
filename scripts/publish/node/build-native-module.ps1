$ErrorActionPreference = "Stop"

$target = $env:TARGET
if (-not $target) { throw "TARGET is required" }
$args = @('--platform', '--release', '--target', $target, '--output-dir', './artifacts')
if ($env:USE_NAPI_CROSS -eq 'true') { $args += '--use-napi-cross' }
if ($env:USE_CROSS -eq 'true') { $args += '--use-cross' }
pnpm --filter html-to-markdown-node exec napi build @args
