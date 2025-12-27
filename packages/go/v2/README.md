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
- Tag format (Go module): `packages/go/vX.Y.Z` (example: `packages/go/v2.15.0`)

```bash
go get github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown
```

## Prerequisites

The Go bindings auto-download a prebuilt `html_to_markdown_ffi` library from GitHub
releases on first use. You can override or disable this behavior if needed:

```bash
# Use a custom library path
export HTML_TO_MARKDOWN_FFI_PATH="/path/to/libhtml_to_markdown_ffi.so"

# Disable downloads (requires HTML_TO_MARKDOWN_FFI_PATH)
export HTML_TO_MARKDOWN_FFI_DISABLE_DOWNLOAD=1

# Override cache location
export HTML_TO_MARKDOWN_FFI_CACHE_DIR="$HOME/.cache/html-to-markdown"

# Override the downloaded version
export HTML_TO_MARKDOWN_FFI_VERSION="2.0.0"
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

## How the binding works

The Go binding calls into the Rust FFI library at runtime via a dynamic loader. On
first use, it downloads a prebuilt `html_to_markdown_ffi` binary from the GitHub
Release that matches your platform, caches it, and then loads it into the process.

Supported platforms (prebuilt):
- linux/amd64
- linux/arm64
- darwin/amd64
- darwin/arm64
- windows/amd64

Release asset naming:
- `html-to-markdown-ffi-<version>-linux-x64.tar.gz`
- `html-to-markdown-ffi-<version>-linux-arm64.tar.gz`
- `html-to-markdown-ffi-<version>-darwin-x64.tar.gz`
- `html-to-markdown-ffi-<version>-darwin-arm64.tar.gz`
- `html-to-markdown-ffi-<version>-windows-x64.zip`

The loader honors environment overrides:
- `HTML_TO_MARKDOWN_FFI_PATH`: use a specific library file (skips download).
- `HTML_TO_MARKDOWN_FFI_DISABLE_DOWNLOAD`: disable downloading (requires `HTML_TO_MARKDOWN_FFI_PATH`).
- `HTML_TO_MARKDOWN_FFI_CACHE_DIR`: override cache directory.
- `HTML_TO_MARKDOWN_FFI_VERSION`: override the release version to download.

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

## Visitor Pattern

The visitor pattern allows you to customize HTML to Markdown conversion by intercepting and modifying the conversion process for specific HTML elements. This enables use cases like custom transformations, content filtering, validation, and analytics.

### Overview

The visitor pattern provides:
- **Node-level customization**: Transform individual HTML elements
- **Fine-grained control**: Implement only the callbacks you need
- **Composable visitors**: Combine multiple visitors for complex transformations
- **Statistics and analytics**: Extract information during conversion
- **Content filtering**: Remove or replace elements based on criteria
- **Format transformation**: Convert elements to custom output

### Core Types

#### VisitResultType

Controls how the converter proceeds after a visitor callback:

```go
const (
	// VisitContinue - use default conversion behavior
	VisitContinue VisitResultType = 0

	// VisitCustom - replace output with CustomOutput field
	VisitCustom VisitResultType = 1

	// VisitSkip - omit element and all children from output
	VisitSkip VisitResultType = 2

	// VisitPreserveHTML - keep original HTML instead of converting
	VisitPreserveHTML VisitResultType = 3

	// VisitError - halt conversion and report error
	VisitError VisitResultType = 4
)
```

#### NodeContext

Information about the current HTML node being visited:

```go
type NodeContext struct {
	NodeType      uint32  // Coarse-grained type classification
	TagName       string  // HTML tag name (e.g., "div", "h1")
	ParentTag     string  // Parent element's tag name
	Depth         uint64  // Depth in DOM tree (0 = root)
	IndexInParent uint64  // Index among siblings (0-based)
	IsInline      bool    // Whether element is inline vs block
}
```

#### VisitResult

Result from a visitor callback:

```go
type VisitResult struct {
	ResultType   VisitResultType  // Action to take
	CustomOutput string           // Custom markdown (if ResultType == VisitCustom)
	ErrorMessage string           // Error message (if ResultType == VisitError)
}
```

### Methods Reference

The `Visitor` struct provides 40+ callback fields for different HTML elements:

#### Generic Callbacks

- `OnElementStart(ctx *NodeContext) *VisitResult` - Before entering any element
- `OnElementEnd(ctx *NodeContext, output string) *VisitResult` - After exiting element
- `OnText(ctx *NodeContext, text string) *VisitResult` - Text nodes (most frequent callback)

#### Content Callbacks

- `OnLink(ctx, href, text, title) *VisitResult` - Links `<a href="...">`
- `OnImage(ctx, src, alt, title) *VisitResult` - Images `<img>`
- `OnHeading(ctx, level, text, id) *VisitResult` - Headings `<h1>-<h6>`
- `OnCodeBlock(ctx, lang, code) *VisitResult` - Code blocks `<pre><code>`
- `OnCodeInline(ctx, code) *VisitResult` - Inline code `<code>`

#### Formatting Callbacks

- `OnStrong(ctx, text) *VisitResult` - Bold `<strong>`, `<b>`
- `OnEmphasis(ctx, text) *VisitResult` - Italic `<em>`, `<i>`
- `OnStrikethrough(ctx, text) *VisitResult` - Strikethrough `<s>`, `<del>`, `<strike>`
- `OnUnderline(ctx, text) *VisitResult` - Underline `<u>`, `<ins>`
- `OnSubscript(ctx, text) *VisitResult` - Subscript `<sub>`
- `OnSuperscript(ctx, text) *VisitResult` - Superscript `<sup>`
- `OnMark(ctx, text) *VisitResult` - Highlight `<mark>`

#### List Callbacks

- `OnListStart(ctx, ordered) *VisitResult` - Before `<ul>` or `<ol>`
- `OnListItem(ctx, ordered, marker, text) *VisitResult` - List items `<li>`
- `OnListEnd(ctx, ordered, output) *VisitResult` - After list

#### Table Callbacks

- `OnTableStart(ctx) *VisitResult` - Before `<table>`
- `OnTableRow(ctx, cells, isHeader) *VisitResult` - Table rows `<tr>`
- `OnTableEnd(ctx, output) *VisitResult` - After table

#### Block Callbacks

- `OnBlockquote(ctx, content, depth) *VisitResult` - Blockquotes `<blockquote>`
- `OnLineBreak(ctx) *VisitResult` - Line breaks `<br>`
- `OnHorizontalRule(ctx) *VisitResult` - Horizontal rules `<hr>`

#### Definition List Callbacks

- `OnDefinitionListStart(ctx) *VisitResult` - Before `<dl>`
- `OnDefinitionTerm(ctx, text) *VisitResult` - Definition terms `<dt>`
- `OnDefinitionDescription(ctx, text) *VisitResult` - Descriptions `<dd>`
- `OnDefinitionListEnd(ctx, output) *VisitResult` - After `</dl>`

#### Form Callbacks

- `OnForm(ctx, action, method) *VisitResult` - Forms `<form>`
- `OnInput(ctx, type, name, value) *VisitResult` - Input fields `<input>`
- `OnButton(ctx, text) *VisitResult` - Buttons `<button>`

#### Media Callbacks

- `OnAudio(ctx, src) *VisitResult` - Audio `<audio>`
- `OnVideo(ctx, src) *VisitResult` - Video `<video>`
- `OnIframe(ctx, src) *VisitResult` - Embedded content `<iframe>`

#### Semantic HTML5 Callbacks

- `OnDetails(ctx, open) *VisitResult` - Collapsible sections `<details>`
- `OnSummary(ctx, text) *VisitResult` - Summary text `<summary>`
- `OnFigureStart(ctx) *VisitResult` - Before `<figure>`
- `OnFigcaption(ctx, text) *VisitResult` - Figure captions `<figcaption>`
- `OnFigureEnd(ctx, output) *VisitResult` - After figure

#### Custom Callbacks

- `OnCustomElement(ctx, tagName, html) *VisitResult` - Unknown or custom elements

### Usage Examples

#### Example 1: Transform Links

```go
html := `<a href="https://example.com">Click here</a>`

visitor := &htmltomarkdown.Visitor{
	OnLink: func(ctx *htmltomarkdown.NodeContext, href, text, title string) *htmltomarkdown.VisitResult {
		// Custom link formatting with URL doubled
		return &htmltomarkdown.VisitResult{
			ResultType: htmltomarkdown.VisitCustom,
			CustomOutput: fmt.Sprintf("[%s](%s) [%s]", text, href, href),
		}
	},
}

markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
// Output: [Click here](https://example.com) [https://example.com]
```

#### Example 2: Filter Content

```go
html := `
<p>Public content</p>
<img src="secret.jpg" alt="Secret image" />
<p>More public content</p>
`

visitor := &htmltomarkdown.Visitor{
	OnImage: func(ctx *htmltomarkdown.NodeContext, src, alt, title string) *htmltomarkdown.VisitResult {
		// Skip all images from output
		return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitSkip}
	},
}

markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
// Images are removed from output
```

#### Example 3: Extract Metadata

```go
html := `<h1>Title</h1><h2>Subtitle</h2><p>Content</p>`

var headings []string

visitor := &htmltomarkdown.Visitor{
	OnHeading: func(ctx *htmltomarkdown.NodeContext, level uint32, text, id string) *htmltomarkdown.VisitResult {
		headings = append(headings, fmt.Sprintf("H%d: %s", level, text))
		return &htmltomarkdown.VisitResult{ResultType: htmltomarkdown.VisitContinue}
	},
}

markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
// headings = ["H1: Title", "H2: Subtitle"]
```

#### Example 4: Remove Formatting

```go
html := `<p>Text with <strong>bold</strong> and <em>italic</em>.</p>`

visitor := &htmltomarkdown.Visitor{
	OnStrong: func(ctx *htmltomarkdown.NodeContext, text string) *htmltomarkdown.VisitResult {
		// Replace bold with plain text
		return &htmltomarkdown.VisitResult{
			ResultType: htmltomarkdown.VisitCustom,
			CustomOutput: text,
		}
	},
	OnEmphasis: func(ctx *htmltomarkdown.NodeContext, text string) *htmltomarkdown.VisitResult {
		// Replace italic with plain text
		return &htmltomarkdown.VisitResult{
			ResultType: htmltomarkdown.VisitCustom,
			CustomOutput: text,
		}
	},
}

markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
// Output: Text with bold and italic.
```

### Visitor Implementation Tips

1. **Only implement needed callbacks** - You don't need to implement all callbacks
2. **Copy string data** - NodeContext strings are only valid during the callback
3. **Use VisitCustom for replacement** - Return custom output to replace element
4. **Use VisitSkip to omit** - Skip elements and their children entirely
5. **Keep text processing fast** - Text callbacks are called 100+ times per document

### Performance Considerations

1. **Minimal Callbacks**: Only implement callbacks you need
2. **Fast Text Processing**: Text callbacks are called frequently (100+ times per document)
3. **Memory Efficiency**: Avoid allocating memory for every callback
4. **Early Returns**: Return quickly from callbacks when possible
5. **String Copies**: Only copy strings if you need to persist them

### Examples

See the `examples/` directory for complete examples:

- `visitor_basic.go` - Basic transformation examples
- `visitor_filter.go` - Content filtering and validation
- `visitor_analytics.go` - Statistics and content extraction

Run examples:

```bash
go run examples/visitor_basic.go
go run examples/visitor_filter.go
go run examples/visitor_analytics.go
```

### API Functions

#### `ConvertWithVisitor(html string, visitor *Visitor) (string, error)`

Converts HTML to Markdown using a custom visitor. The visitor allows you to intercept and customize the conversion process for specific HTML elements.

```go
markdown, err := htmltomarkdown.ConvertWithVisitor(html, visitor)
if err != nil {
	log.Fatal(err)
}
```

#### `MustConvertWithVisitor(html string, visitor *Visitor) string`

Like `ConvertWithVisitor` but panics if an error occurs.

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
| Lists (Timeline)       | 129 KB | 2,197    | 277.5 MB/s |
| Tables (Countries)     | 360 KB | 745      | 262.1 MB/s |
| Medium (Python)        | 656 KB | 371      | 237.9 MB/s |
| Large (Rust)           | 567 KB | 384      | 212.8 MB/s |
| Small (Intro)          | 463 KB | 580      | 262.6 MB/s |
| HOCR German PDF        | 44 KB  | 2,598    | 110.8 MB/s |
| HOCR Invoice           | 4 KB   | 5,583    | 22.8 MB/s  |
| HOCR Embedded Tables   | 37 KB  | 2,187    | 79.4 MB/s  |

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
