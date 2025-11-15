# html-to-markdown Go Bindings

High-performance HTML to Markdown converter with Go bindings to the Rust core library.

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
