# html-to-markdown C# Bindings

High-performance HTML to Markdown converter with C# bindings to the Rust core library.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown.svg)](https://crates.io/crates/html-to-markdown)
[![npm (node)](https://badge.fury.io/js/html-to-markdown-node.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://badge.fury.io/js/html-to-markdown-wasm.svg)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/Goldziher.HtmlToMarkdown.svg)](https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

## Installation

> **NuGet package ID**
>
> NuGet package names are global. To avoid clashing with an older community package named `HtmlToMarkdown`, our official bindings are published as **`Goldziher.HtmlToMarkdown`**. Use that ID in all `dotnet` CLI commands.

```bash
dotnet add package Goldziher.HtmlToMarkdown
```

## Prerequisites

The native library `html_to_markdown_ffi` must be available:

### Windows
```bash
cargo build --release -p html-to-markdown-ffi
copy target\release\html_to_markdown_ffi.dll %WINDIR%\System32\
```

### Linux
```bash
cargo build --release -p html-to-markdown-ffi
sudo cp target/release/libhtml_to_markdown_ffi.so /usr/local/lib/
sudo ldconfig
```

### macOS
```bash
cargo build --release -p html-to-markdown-ffi
sudo cp target/release/libhtml_to_markdown_ffi.dylib /usr/local/lib/
```

## Usage

```csharp
using HtmlToMarkdown;

var html = "<h1>Hello World</h1><p>This is a paragraph.</p>";

try
{
    var markdown = HtmlToMarkdownConverter.Convert(html);
    Console.WriteLine(markdown);
}
catch (HtmlToMarkdownException ex)
{
    Console.Error.WriteLine($"Conversion failed: {ex.Message}");
}
```

## API

### `HtmlToMarkdownConverter.Convert(string html)`

Converts HTML to Markdown. Throws `HtmlToMarkdownException` on error.

### `HtmlToMarkdownConverter.GetVersion()`

Returns the library version string.

## Testing

```bash
cd packages/csharp
dotnet test HtmlToMarkdown.Tests/HtmlToMarkdown.Tests.csproj
```

## Performance

The Rust-backed implementation provides excellent performance:

| Document Type          | Size   | Ops/sec  | Throughput |
| ---------------------- | ------ | -------- | ---------- |
| Lists (Timeline)       | 129 KB | 1,351    | 170.6 MB/s |
| Tables (Countries)     | 360 KB | 322      | 113.3 MB/s |
| Medium (Python)        | 656 KB | 163      | 104.5 MB/s |
| Large (Rust)           | 567 KB | 180      | 99.9 MB/s  |
| Small (Intro)          | 463 KB | 184      | 83.3 MB/s  |
| HOCR German PDF        | 44 KB  | 2,667    | 113.8 MB/s |
| HOCR Invoice           | 4 KB   | 27,795   | 113.7 MB/s |
| HOCR Embedded Tables   | 37 KB  | 2,933    | 106.5 MB/s |

## Publishing to NuGet

### 1. Build the package

```bash
cd packages/csharp/HtmlToMarkdown
dotnet pack --configuration Release
```

### 2. Get NuGet API Key

1. Create account at [nuget.org](https://www.nuget.org/)
2. Go to Account → API Keys
3. Create new API key with push permissions

### 3. Publish

```bash
dotnet nuget push bin/Release/Goldziher.HtmlToMarkdown.2.8.0.nupkg \
    --api-key YOUR_API_KEY \
    --source https://api.nuget.org/v3/index.json
```

### 4. Verify

Check your package at: https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/

## License

MIT
