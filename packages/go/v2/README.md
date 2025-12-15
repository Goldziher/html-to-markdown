# html-to-markdown Go Bindings

High-performance HTML to Markdown converter with Go bindings to the Rust core library.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg?logo=rust&label=crates.io)](https://crates.io/crates/html-to-markdown-rs)
[![npm (node)](https://img.shields.io/npm/v/html-to-markdown-node.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://img.shields.io/npm/v/html-to-markdown-wasm.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://img.shields.io/pypi/v/html-to-markdown.svg?logo=pypi)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/Goldziher.HtmlToMarkdown.svg)](https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

## Installation

This module uses semantic import versioning (`/v2`) and lives in a subdirectory.

- Import path: `github.com/Goldziher/html-to-markdown/packages/go/v2`
- Tag format (Go module): `packages/go/v2/vX.Y.Z` (example: `packages/go/v2/v2.14.8`)

```bash
go get github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown
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
export DYLD_LIBRARY_PATH=$PWD/target/release:$DYLD_LIBRARY_PATH
export CGO_LDFLAGS="-L$PWD/target/release $CGO_LDFLAGS"
```

## Usage

```go
package main

import (
    "fmt"
    "log"

    "github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown"
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

### `ConvertWithMetadata(html string) (MetadataExtraction, error)`

Converts HTML to Markdown while extracting comprehensive metadata including document information, headers, links, images, and structured data.

Returns a `MetadataExtraction` struct containing both the converted markdown and extracted metadata, or an error if conversion fails.

## Metadata Types

The metadata extraction API provides rich, typed access to extracted document information:

### `ExtendedMetadata`

The top-level metadata container with all extracted information:

```go
type ExtendedMetadata struct {
    Document       DocumentMetadata  // Document-level metadata
    Headers        []HeaderMetadata  // Extracted headers (h1-h6)
    Links          []LinkMetadata    // Extracted hyperlinks
    Images         []ImageMetadata   // Extracted images
    StructuredData []StructuredData  // JSON-LD, Microdata, RDFa blocks
}
```

### `DocumentMetadata`

Document-level metadata from `<head>` and top-level elements:

```go
type DocumentMetadata struct {
    Title           *string            // <title> content
    Description     *string            // meta[name="description"]
    Keywords        []string           // meta[name="keywords"], split on commas
    Author          *string            // meta[name="author"]
    CanonicalURL    *string            // <link rel="canonical">
    BaseHref        *string            // <base href="">
    Language        *string            // lang attribute
    TextDirection   *TextDirection     // dir attribute (ltr, rtl, auto)
    OpenGraph       map[string]string  // og:* properties
    TwitterCard     map[string]string  // twitter:* properties
    MetaTags        map[string]string  // Other meta tags
}
```

### `HeaderMetadata`

Header element metadata with hierarchy tracking:

```go
type HeaderMetadata struct {
    Level       uint8   // Header level (1 for h1, 6 for h6)
    Text        string  // Normalized text content
    ID          *string // HTML id attribute
    Depth       uint32  // Document tree depth
    HTMLOffset  uint32  // Byte offset in original HTML
}
```

### `LinkMetadata`

Hyperlink metadata with classification:

```go
type LinkMetadata struct {
    Href       string            // href attribute value
    Text       string            // Link text content
    Title      *string           // title attribute
    LinkType   LinkType          // Classification (anchor, internal, external, email, phone, other)
    Rel        []string          // rel attribute values
    Attributes map[string]string // Additional HTML attributes
}
```

Link types:
- `LinkTypeAnchor` - Fragment links (`#anchor`)
- `LinkTypeInternal` - Same-domain links
- `LinkTypeExternal` - Cross-domain links
- `LinkTypeEmail` - mailto: links
- `LinkTypePhone` - tel: links
- `LinkTypeOther` - Other protocols or unclassifiable

### `ImageMetadata`

Image metadata with source and dimensions:

```go
type ImageMetadata struct {
    Src        string            // Image source (URL, data URI, or SVG identifier)
    Alt        *string           // alt attribute (accessibility)
    Title      *string           // title attribute
    Dimensions *[2]uint32        // [width, height] if available
    ImageType  ImageType         // Classification
    Attributes map[string]string // Additional HTML attributes
}
```

Image types:
- `ImageTypeDataURI` - Data URIs (base64)
- `ImageTypeInlineSVG` - Inline SVG elements
- `ImageTypeExternal` - External URLs (http/https)
- `ImageTypeRelative` - Relative paths

### `StructuredData`

Machine-readable structured data blocks:

```go
type StructuredData struct {
    DataType   StructuredDataType // json_ld, microdata, rdfa
    RawJSON    string             // Raw JSON string (for JSON-LD)
    SchemaType *string            // Schema type (e.g., "Article", "Event")
}
```

## Metadata Examples

### Basic Metadata Extraction

Extract and access document metadata:

```go
package main

import (
    "fmt"
    "log"

    "github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown"
)

func main() {
    html := `<html>
        <head>
            <title>My Article</title>
            <meta name="description" content="An interesting read">
            <meta name="author" content="Jane Doe">
        </head>
        <body>
            <h1>Welcome</h1>
            <p>Content here.</p>
        </body>
    </html>`

    result, err := htmltomarkdown.ConvertWithMetadata(html)
    if err != nil {
        log.Fatal(err)
    }

    // Access document metadata
    if result.Metadata.Document.Title != nil {
        fmt.Printf("Title: %s\n", *result.Metadata.Document.Title)
    }
    if result.Metadata.Document.Author != nil {
        fmt.Printf("Author: %s\n", *result.Metadata.Document.Author)
    }

    // Count extracted elements
    fmt.Printf("Headers: %d\n", len(result.Metadata.Headers))
    fmt.Printf("Links: %d\n", len(result.Metadata.Links))
    fmt.Printf("Images: %d\n", len(result.Metadata.Images))

    // Print markdown
    fmt.Println("\nMarkdown:")
    fmt.Println(result.Markdown)
}
```

### Processing Headers with Hierarchy

Extract and traverse document headers:

```go
package main

import (
    "fmt"
    "log"

    "github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown"
)

func main() {
    html := `<html><body>
        <h1>Main Title</h1>
        <h2>Section One</h2>
        <h3>Subsection</h3>
        <h2>Section Two</h2>
    </body></html>`

    result, err := htmltomarkdown.ConvertWithMetadata(html)
    if err != nil {
        log.Fatal(err)
    }

    // Traverse headers by level
    fmt.Println("Document Structure:")
    for _, header := range result.Metadata.Headers {
        indent := ""
        for i := uint8(1); i < header.Level; i++ {
            indent += "  "
        }
        fmt.Printf("%s- Level %d: %s", indent, header.Level, header.Text)
        if header.ID != nil {
            fmt.Printf(" (#%s)", *header.ID)
        }
        fmt.Println()
    }
}
```

### Analyzing Links with Type Classification

Extract and filter links by type:

```go
package main

import (
    "fmt"
    "log"

    "github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown"
)

func main() {
    html := `<html><body>
        <a href="https://external.com">External</a>
        <a href="/internal">Internal</a>
        <a href="#section">Anchor</a>
        <a href="mailto:test@example.com">Email</a>
    </body></html>`

    result, err := htmltomarkdown.ConvertWithMetadata(html)
    if err != nil {
        log.Fatal(err)
    }

    // Categorize links by type
    linksByType := make(map[htmltomarkdown.LinkType][]htmltomarkdown.LinkMetadata)
    for _, link := range result.Metadata.Links {
        linksByType[link.LinkType] = append(linksByType[link.LinkType], link)
    }

    // Process external links
    if external, found := linksByType[htmltomarkdown.LinkTypeExternal]; found {
        fmt.Printf("Found %d external links:\n", len(external))
        for _, link := range external {
            fmt.Printf("  - %s: %s\n", link.Text, link.Href)
        }
    }

    // Check for nofollow links
    fmt.Println("\nNofollow links:")
    for _, link := range result.Metadata.Links {
        for _, rel := range link.Rel {
            if rel == "nofollow" {
                fmt.Printf("  - %s: %s\n", link.Text, link.Href)
            }
        }
    }
}
```

## Memory Management Notes

The Go bindings handle C FFI memory automatically:

1. **String Conversion**: Go automatically manages conversion between Go strings and C strings using `cgo`
2. **Deferred Cleanup**: Both markdown and metadata JSON pointers are freed via `defer` statements
3. **JSON Parsing**: Metadata JSON is unmarshaled once into typed Go structs, allowing garbage collection of the raw JSON
4. **Error Handling**: Memory is properly cleaned up even when errors occur

No manual memory management is required - Go's runtime handles all C memory deallocation.

## Testing

```bash
cd packages/go/v2/htmltomarkdown
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
import "github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown"
```

To use a specific version:

```bash
go get github.com/Goldziher/html-to-markdown/packages/go/v2@v2.14.8
```

## License

MIT
