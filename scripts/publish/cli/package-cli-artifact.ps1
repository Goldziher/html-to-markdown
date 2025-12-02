$ErrorActionPreference = "Stop"

$target = $env:TARGET
if (-not $target) { throw "TARGET is required" }
$stage = "cli-$target"
Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $stage | Out-Null
Copy-Item "target/$target/release/html-to-markdown.exe" $stage
Copy-Item LICENSE $stage
Copy-Item README.md $stage
Compress-Archive -Path "$stage/*" -DestinationPath "$stage.zip" -Force
Remove-Item -Recurse -Force $stage
