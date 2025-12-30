# html-to-markdown

<div align="center" style="display: flex; flex-wrap: wrap; gap: 8px; justify-content: center; margin: 20px 0;">
  <!-- Language Bindings -->
  <a href="https://crates.io/crates/html-to-markdown-rs">
    <img src="https://img.shields.io/crates/v/html-to-markdown-rs?label=Rust&color=007ec6" alt="Rust">
  </a>
  <a href="https://pypi.org/project/html-to-markdown/">
    <img src="https://img.shields.io/pypi/v/html-to-markdown?label=Python&color=007ec6" alt="Python">
  </a>
  <a href="https://www.npmjs.com/package/@kreuzberg/html-to-markdown-node">
    <img src="https://img.shields.io/npm/v/@kreuzberg/html-to-markdown-node?label=Node.js&color=007ec6" alt="Node.js">
  </a>
  <a href="https://www.npmjs.com/package/@kreuzberg/html-to-markdown-wasm">
    <img src="https://img.shields.io/npm/v/@kreuzberg/html-to-markdown-wasm?label=WASM&color=007ec6" alt="WASM">
  </a>
  <a href="https://central.sonatype.com/artifact/dev.kreuzberg/html-to-markdown">
    <img src="https://img.shields.io/maven-central/v/dev.kreuzberg/html-to-markdown?label=Java&color=007ec6" alt="Java">
  </a>
  <a href="https://pkg.go.dev/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown">
    <img src="https://img.shields.io/badge/Go-v2.19.1-007ec6" alt="Go">
  </a>
  <a href="https://www.nuget.org/packages/KreuzbergDev.HtmlToMarkdown/">
    <img src="https://img.shields.io/nuget/v/KreuzbergDev.HtmlToMarkdown?label=C%23&color=007ec6" alt="C#">
  </a>
  <a href="https://packagist.org/packages/goldziher/html-to-markdown">
    <img src="https://img.shields.io/packagist/v/goldziher/html-to-markdown?label=PHP&color=007ec6" alt="PHP">
  </a>
  <a href="https://rubygems.org/gems/html-to-markdown">
    <img src="https://img.shields.io/gem/v/html-to-markdown?label=Ruby&color=007ec6" alt="Ruby">
  </a>
  <a href="https://hex.pm/packages/html_to_markdown">
    <img src="https://img.shields.io/hexpm/v/html_to_markdown?label=Elixir&color=007ec6" alt="Elixir">
  </a>

  <!-- Project Info -->
  <a href="https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  </a>
</div>

<img width="1128" height="191" alt="html-to-markdown" src="https://github.com/user-attachments/assets/419fc06c-8313-4324-b159-4b4d3cfce5c0" />

<div align="center" style="margin-top: 20px;">
  <a href="https://discord.gg/pXxagNK2zN">
      <img height="22" src="https://img.shields.io/badge/Discord-Join%20our%20community-7289da?logo=discord&logoColor=white" alt="Discord">
  </a>
</div>


High-performance HTML to Markdown converter with Go bindings to the Rust core library.
Supports automatic downloading of prebuilt FFI libraries for Linux, macOS, and Windows with customizable caching.


## Installation

```bash
go get github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown
```



Requires Go 1.25+. The FFI library is automatically downloaded from GitHub releases.

## FFI Library Configuration

The Go bindings use a C FFI library compiled from Rust. By default, the library is automatically downloaded from [GitHub releases](https://github.com/kreuzberg-dev/html-to-markdown/releases).

### Environment Variables

Control FFI library loading with these environment variables:

- **`HTML_TO_MARKDOWN_FFI_VERSION`** – Override the default FFI version (default: `2.19.1`)
  ```bash
  export HTML_TO_MARKDOWN_FFI_VERSION=2.19.0
  ```

- **`HTML_TO_MARKDOWN_FFI_PATH`** – Use a local FFI library instead of downloading
  ```bash
  export HTML_TO_MARKDOWN_FFI_PATH=/path/to/libhtml_to_markdown_ffi.so
  ```

- **`HTML_TO_MARKDOWN_FFI_CACHE_DIR`** – Customize where libraries are cached (default: `$HOME/.cache/html-to-markdown/ffi`)
  ```bash
  export HTML_TO_MARKDOWN_FFI_CACHE_DIR=/custom/cache/path
  ```

- **`HTML_TO_MARKDOWN_FFI_DISABLE_DOWNLOAD`** – Disable automatic downloads (useful in offline environments)
  ```bash
  export HTML_TO_MARKDOWN_FFI_DISABLE_DOWNLOAD=1
  ```

### Development

For local development, build the FFI library and point to it:

```bash
# Build FFI library
cargo build -p html-to-markdown-ffi --release

# Find the library path (platform-dependent)
# Linux: target/release/libhtml_to_markdown_ffi.so
# macOS: target/release/libhtml_to_markdown_ffi.dylib
# Windows: target/release/html_to_markdown_ffi.dll

export HTML_TO_MARKDOWN_FFI_PATH=$(pwd)/target/release/libhtml_to_markdown_ffi.so
go test ./...
```

### Troubleshooting

**"404 Not Found" error:** The default version may be outdated or not published. Options:

1. Update to the latest version:
   ```bash
   export HTML_TO_MARKDOWN_FFI_VERSION=2.19.1
   ```

2. Check [available releases](https://github.com/kreuzberg-dev/html-to-markdown/releases)

3. Build and use a local library (see Development section above)






## Performance Snapshot

Apple M4 • Real Wikipedia documents • `Convert()` (Go)

| Document | Size | Latency | Throughput |
| -------- | ---- | ------- | ---------- |
| Lists (Timeline) | 129KB | 0.46ms | 277.5 MB/s |
| Tables (Countries) | 360KB | 1.37ms | 262.1 MB/s |
| Mixed (Python wiki) | 656KB | 2.75ms | 237.9 MB/s |


See [Performance Guide](../../examples/performance/) for detailed benchmarks.


## Quick Start

Basic conversion:

```go
package main

import (
    "fmt"
    "log"

    "github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown"
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



With conversion options:

```go
package main

import (
    "fmt"
    "log"

    "github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown"
)

func main() {
    // Check library version
    version := htmltomarkdown.Version()
    fmt.Printf("html-to-markdown version: %s\n", version)

    html := "<h1>Hello</h1><p>Welcome</p>"

    // Convert with error handling
    markdown, err := htmltomarkdown.Convert(html)
    if err != nil {
        log.Fatalf("Conversion failed: %v", err)
    }

    fmt.Println(markdown)

    // Alternative: Use MustConvert for panicking on error
    // Useful when you're certain conversion won't fail
    anotherMarkdown := htmltomarkdown.MustConvert("<p>Safe HTML</p>")
    fmt.Println(anotherMarkdown)
}
```






## API Reference

### Core Functions


**`Convert(html string, options *ConversionOptions) (string, error)`**

Basic HTML-to-Markdown conversion. Fast and simple.

**`ConvertWithMetadata(html string, options *ConversionOptions, config *MetadataConfig) (string, Metadata, error)`**

Extract Markdown plus metadata in a single pass. See [Metadata Extraction Guide](../../examples/metadata-extraction/).

**`ConvertWithInlineImages(html string, config *InlineImageConfig) (string, []ImageData, []string, error)`**

Extract base64-encoded inline images with metadata.



### Options

**`ConversionOptions`** – Key configuration fields:
- `heading_style`: Heading format (`"underlined"` | `"atx"` | `"atx_closed"`) — default: `"underlined"`
- `list_indent_width`: Spaces per indent level — default: `2`
- `bullets`: Bullet characters cycle — default: `"*+-"`
- `wrap`: Enable text wrapping — default: `false`
- `wrap_width`: Wrap at column — default: `80`
- `code_language`: Default fenced code block language — default: none
- `extract_metadata`: Embed metadata as YAML frontmatter — default: `false`

**`MetadataConfig`** – Selective metadata extraction:
- `extract_headers`: h1-h6 elements — default: `true`
- `extract_links`: Hyperlinks — default: `true`
- `extract_images`: Image elements — default: `true`
- `extract_structured_data`: JSON-LD, Microdata, RDFa — default: `true`
- `max_structured_data_size`: Size limit in bytes — default: `100KB`






## Examples

- [Visitor Pattern Guide](../../examples/visitor-pattern/)
- [Metadata Extraction Guide](../../examples/metadata-extraction/)
- [Performance Guide](../../examples/performance/)

## Links

- **GitHub:** [github.com/kreuzberg-dev/html-to-markdown](https://github.com/kreuzberg-dev/html-to-markdown)

- **Go Packages:** [pkg.go.dev/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2](https://pkg.go.dev/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2)

- **Kreuzberg Ecosystem:** [kreuzberg.dev](https://kreuzberg.dev)
- **Discord:** [discord.gg/pXxagNK2zN](https://discord.gg/pXxagNK2zN)

## Contributing

We welcome contributions! Please see our [Contributing Guide](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/CONTRIBUTING.md) for details on:

- Setting up the development environment
- Running tests locally
- Submitting pull requests
- Reporting issues

All contributions must follow our code quality standards (enforced via pre-commit hooks):

- Proper test coverage (Rust 95%+, language bindings 80%+)
- Formatting and linting checks
- Documentation for public APIs

## License

MIT License – see [LICENSE](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE).

## Support

If you find this library useful, consider [sponsoring the project](https://github.com/sponsors/kreuzberg-dev).

Have questions or run into issues? We're here to help:

- **GitHub Issues:** [github.com/kreuzberg-dev/html-to-markdown/issues](https://github.com/kreuzberg-dev/html-to-markdown/issues)
- **Discussions:** [github.com/kreuzberg-dev/html-to-markdown/discussions](https://github.com/kreuzberg-dev/html-to-markdown/discussions)
- **Discord Community:** [discord.gg/pXxagNK2zN](https://discord.gg/pXxagNK2zN)
