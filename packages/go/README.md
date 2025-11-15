# html-to-markdown Go Bindings

High-performance HTML to Markdown converter with Go bindings to the Rust core library.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown.svg)](https://crates.io/crates/html-to-markdown)
[![npm (node)](https://badge.fury.io/js/html-to-markdown-node.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://badge.fury.io/js/html-to-markdown-wasm.svg)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/HtmlToMarkdown.svg)](https://www.nuget.org/packages/HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

## Installation

```bash
go get github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown
```

## Prerequisites

The native library `libhtml_to_markdown_ffi` must be available:

```bash
# Build the FFI library
cargo build --release -p html-to-markdown-ffi

# Copy to system library path (Linux/macOS)
sudo cp target/release/libhtml_to_markdown_ffi.* /usr/local/lib/

# Or set LD_LIBRARY_PATH (Linux) / DYLD_LIBRARY_PATH (macOS)
export LD_LIBRARY_PATH=$PWD/target/release:$LD_LIBRARY_PATH
```

## Usage

```go
package main

import (
    "fmt"
    "log"

    "github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown"
)

func main() {
    html := "<h1>Hello World</h1><p>This is a paragraph.</p>"

    markdown, err := htmltomarkdown.Convert(html)
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println(markdown)
}
```

## API

### `Convert(html string) (string, error)`

Converts HTML to Markdown. Returns an error if conversion fails.

### `MustConvert(html string) string`

Like `Convert` but panics on error. Useful when errors are unexpected.

### `Version() string`

Returns the library version string.

## Testing

```bash
cd packages/go/htmltomarkdown
go test -v
go test -bench=.
```

## Performance

The Rust-backed implementation provides excellent performance:

| Document Type          | Size   | Ops/sec  | Throughput |
| ---------------------- | ------ | -------- | ---------- |
| Lists (Timeline)       | 129 KB | 1,306    | 165.0 MB/s |
| Tables (Countries)     | 360 KB | 330      | 116.1 MB/s |
| Medium (Python)        | 656 KB | 151      | 96.9 MB/s  |
| Large (Rust)           | 567 KB | 165      | 91.4 MB/s  |
| Small (Intro)          | 463 KB | 201      | 91.1 MB/s  |
| HOCR German PDF        | 44 KB  | 2,542    | 108.4 MB/s |
| HOCR Invoice           | 4 KB   | 26,369   | 107.9 MB/s |
| HOCR Embedded Tables   | 37 KB  | 2,765    | 100.4 MB/s |

## Publishing

Go packages are published by pushing to GitHub. Users import directly:

```go
import "github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown"
```

To use a specific version:

```bash
go get github.com/Goldziher/html-to-markdown/packages/go@v2.8.0
```

## License

MIT
