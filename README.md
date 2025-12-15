# html-to-markdown

High-performance HTML → Markdown conversion powered by Rust. Shipping as a Rust crate, Python package, PHP extension, Ruby gem, Elixir Rustler NIF, Node.js bindings, WebAssembly, and standalone CLI with identical rendering behaviour.

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

---

## 🎮 **[Try the Live Demo →](https://goldziher.github.io/html-to-markdown/)**

Experience WebAssembly-powered HTML to Markdown conversion instantly in your browser. No installation needed!

---

## Why html-to-markdown?

- **Blazing Fast**: Rust-powered core delivers 10-80× faster conversion than pure Python alternatives
- **Universal**: Works everywhere - Node.js, Bun, Deno, browsers, Python, Rust, and standalone CLI
- **Smart Conversion**: Handles complex documents including nested tables, code blocks, task lists, and hOCR OCR output
- **Metadata Extraction**: Extract document metadata (title, description, headers, links, images) alongside conversion
- **Highly Configurable**: Control heading styles, code block fences, list formatting, whitespace handling, and HTML sanitization
- **Tag Preservation**: Keep specific HTML tags unconverted when markdown isn't expressive enough
- **Secure by Default**: Built-in HTML sanitization prevents malicious content
- **Consistent Output**: Identical markdown rendering across all language bindings

## Documentation

**Language Guides & API References:**

- **Python** – [README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/python/README.md) with metadata extraction, inline images, hOCR workflows
- **JavaScript/TypeScript** – [Node.js](https://github.com/Goldziher/html-to-markdown/blob/main/crates/html-to-markdown-node/README.md) | [TypeScript](https://github.com/Goldziher/html-to-markdown/blob/main/packages/typescript/README.md) | [WASM](https://github.com/Goldziher/html-to-markdown/blob/main/crates/html-to-markdown-wasm/README.md)
- **Ruby** – [README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/ruby/README.md) with RBS types, Steep type checking
- **PHP** – [Package](https://github.com/Goldziher/html-to-markdown/blob/main/packages/php/README.md) | [Extension (PIE)](https://github.com/Goldziher/html-to-markdown/blob/main/packages/php-ext/README.md)
- **Go** – [README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/go/README.md) with FFI bindings
- **Java** – [README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/java/README.md) with Panama FFI, Maven/Gradle setup
- **C#/.NET** – [README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/csharp/README.md) with NuGet distribution
- **Elixir** – [README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/elixir/README.md) with Rustler NIF bindings
- **Rust** – [README](https://github.com/Goldziher/html-to-markdown/blob/main/crates/html-to-markdown/README.md) with core API, error handling, advanced features

**Project Resources:**

- **Contributing** – [CONTRIBUTING.md](https://github.com/Goldziher/html-to-markdown/blob/main/CONTRIBUTING.md) ⭐ Start here for development
- **Changelog** – [CHANGELOG.md](https://github.com/Goldziher/html-to-markdown/blob/main/CHANGELOG.md) – Version history and breaking changes

## Installation

| Target                      | Command(s)                                                                                                       |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| **Node.js/Bun** (native)    | `npm install html-to-markdown-node`                                                                              |
| **WebAssembly** (universal) | `npm install html-to-markdown-wasm`                                                                              |
| **Deno**                    | `import { convert } from "npm:html-to-markdown-wasm"`                                                            |
| **Python** (bindings + CLI) | `pip install html-to-markdown`                                                                                   |
| **PHP** (extension + helpers) | `PHP_EXTENSION_DIR=$(php-config --extension-dir) pie install goldziher/html-to-markdown`<br>`composer require goldziher/html-to-markdown` |
| **Ruby** gem                | `bundle add html-to-markdown` or `gem install html-to-markdown`                                                  |
| **Elixir** (Rustler NIF)    | `{:html_to_markdown, "~> 2.8"}`                                                                                  |
| **Rust** crate              | `cargo add html-to-markdown-rs`                                                                                  |
| Rust CLI (crates.io)        | `cargo install html-to-markdown-cli`                                                                             |
| Homebrew CLI                | `brew install html-to-markdown` (core)                                                                          |
| Releases                    | [GitHub Releases](https://github.com/Goldziher/html-to-markdown/releases)                                        |

## Quick Start

### JavaScript/TypeScript

**Node.js / Bun (Native - Fastest):**

```typescript
import { convert } from 'html-to-markdown-node';

const html = '<h1>Hello</h1><p>Rust ❤️ Markdown</p>';
const markdown = convert(html, {
  headingStyle: 'Atx',
  codeBlockStyle: 'Backticks',
  wrap: true,
  preserveTags: ['table'], // NEW in v2.5: Keep complex HTML as-is
});
```

**Deno / Browsers / Edge (Universal):**

```typescript
import { convert } from "npm:html-to-markdown-wasm"; // Deno
// or: import { convert } from 'html-to-markdown-wasm'; // Bundlers

const markdown = convert(html, {
  headingStyle: 'atx',
  listIndentWidth: 2,
});
```

**Performance:** The shared fixture harness (`task bench:bindings`) now clocks C# at ~1.4k ops/sec (≈171 MB/s), Go at ~1.3k ops/sec (≈165 MB/s), Node, Python, and the Rust CLI at ~1.3–1.4k ops/sec (≈150 MB/s) on the 129 KB Wikipedia "Lists" page thanks to the new Buffer/Uint8Array fast paths and release-mode harness. Ruby stays close at ~1.2k ops/sec (≈150 MB/s), Java lands at ~1.0k ops/sec (≈126 MB/s), WASM hits ~0.85k ops/sec (≈108 MB/s), and PHP achieves ~0.3k ops/sec (≈35 MB/s)—all providing excellent throughput for production workloads.

See the JavaScript guides for full API documentation:

- [Node.js/Bun guide](https://github.com/Goldziher/html-to-markdown/tree/main/crates/html-to-markdown-node)
- [WebAssembly guide](https://github.com/Goldziher/html-to-markdown/tree/main/crates/html-to-markdown-wasm)

### Metadata extraction (all languages)

```typescript
import { convertWithMetadata } from 'html-to-markdown-node';

const html = `
  <html>
    <head>
      <title>Example</title>
      <meta name="description" content="Demo page">
      <link rel="canonical" href="https://example.com/page">
    </head>
    <body>
      <h1 id="welcome">Welcome</h1>
      <a href="https://example.com" rel="nofollow external">Example link</a>
      <img src="https://example.com/image.jpg" alt="Hero" width="640" height="480">
    </body>
  </html>
`;

const { markdown, metadata } = await convertWithMetadata(
  html,
  { headingStyle: 'Atx' },
  { extract_links: true, extract_images: true, extract_headers: true },
);

console.log(markdown);
// metadata.document.title === 'Example'
// metadata.links[0].rel === ['nofollow', 'external']
// metadata.images[0].dimensions === [640, 480]
```

Equivalent APIs are available in every binding:

- Python: `convert_with_metadata(html, options=None, metadata_config=None)`
- Ruby: `HtmlToMarkdown.convert_with_metadata(html, options = nil, metadata_config = nil)`
- PHP: `convert_with_metadata(string $html, ?array $options = null, ?array $metadataConfig = null)`

### CLI

```bash
# Convert a file
html-to-markdown input.html > output.md

# Stream from stdin
curl https://example.com | html-to-markdown > output.md

# Apply options
html-to-markdown --heading-style atx --list-indent-width 2 input.html

# Fetch a remote page (HTTP) with optional custom User-Agent
html-to-markdown --url https://example.com > output.md
html-to-markdown --url https://example.com --user-agent "Mozilla/5.0" > output.md
```

### Metadata Extraction

Extract document metadata alongside HTML-to-Markdown conversion. All bindings support identical APIs:

#### CLI Examples

```bash
# Basic metadata extraction with conversion
html-to-markdown input.html --with-metadata -o output.json

# Extract document metadata (title, description, language, etc.)
html-to-markdown input.html --with-metadata --extract-document

# Extract headers and links
html-to-markdown input.html --with-metadata --extract-headers --extract-links

# Extract all metadata types with conversion
html-to-markdown input.html --with-metadata \
  --extract-document \
  --extract-headers \
  --extract-links \
  --extract-images \
  --extract-structured-data \
  -o metadata.json

# Fetch and extract from remote URL
html-to-markdown --url https://example.com --with-metadata -o output.json

# Web scraping with preprocessing and metadata
html-to-markdown page.html --preprocess --preset aggressive \
  --with-metadata --extract-links --extract-images
```

Output format (JSON):

```json
{
  "markdown": "# Title\n\nContent here...",
  "metadata": {
    "document": {
      "title": "Page Title",
      "description": "Meta description",
      "charset": "utf-8",
      "language": "en"
    },
    "headers": [
      { "level": 1, "text": "Title", "id": "title" }
    ],
    "links": [
      {
        "text": "Example",
        "href": "https://example.com",
        "title": null,
        "rel": ["external"]
      }
    ],
    "images": [
      {
        "src": "https://example.com/image.jpg",
        "alt": "Hero image",
        "title": null,
        "dimensions": [640, 480]
      }
    ]
  }
}
```

#### Python Example

```python
from html_to_markdown import convert_with_metadata

html = '''
<html>
  <head>
    <title>Product Guide</title>
    <meta name="description" content="Complete product documentation">
  </head>
  <body>
    <h1>Getting Started</h1>
    <p>Visit our <a href="https://example.com">website</a> for more.</p>
    <img src="https://example.com/guide.jpg" alt="Setup diagram" width="800" height="600">
  </body>
</html>
'''

markdown, metadata = convert_with_metadata(
    html,
    options={'heading_style': 'Atx'},
    metadata_config={
        'extract_document': True,
        'extract_headers': True,
        'extract_links': True,
        'extract_images': True,
    }
)

print(markdown)
print(f"Title: {metadata['document']['title']}")
print(f"Links found: {len(metadata['links'])}")
```

#### TypeScript/Node.js Example

```typescript
import { convertWithMetadata } from 'html-to-markdown-node';

const html = `
  <html>
    <head>
      <title>Article</title>
      <meta name="description" content="Tech article">
    </head>
    <body>
      <h1>Web Performance</h1>
      <p>Read our <a href="/blog">blog</a> for tips.</p>
      <img src="/perf.png" alt="Chart" width="1200" height="630">
    </body>
  </html>
`;

const { markdown, metadata } = await convertWithMetadata(html, {
  headingStyle: 'Atx',
}, {
  extract_document: true,
  extract_headers: true,
  extract_links: true,
  extract_images: true,
});

console.log(markdown);
console.log(`Found ${metadata.headers.length} headers`);
console.log(`Found ${metadata.links.length} links`);
```

#### Ruby Example

```ruby
require 'html_to_markdown'

html = <<~HTML
  <html>
    <head>
      <title>Documentation</title>
      <meta name="description" content="API Reference">
    </head>
    <body>
      <h2>Installation</h2>
      <p>See our <a href="https://github.com">GitHub</a>.</p>
      <img src="https://example.com/diagram.svg" alt="Architecture" width="960" height="540">
    </body>
  </html>
HTML

markdown, metadata = HtmlToMarkdown.convert_with_metadata(
  html,
  options: { heading_style: :atx },
  metadata_config: {
    extract_document: true,
    extract_headers: true,
    extract_links: true,
    extract_images: true,
  }
)

puts markdown
puts "Title: #{metadata[:document][:title]}"
puts "Images: #{metadata[:images].length}"
```

#### PHP Example

```php
<?php
use HtmlToMarkdown\HtmlToMarkdown;

$html = <<<HTML
<html>
  <head>
    <title>Tutorial</title>
    <meta name="description" content="Step-by-step guide">
  </head>
  <body>
    <h1>Getting Started</h1>
    <p>Check our <a href="https://example.com/guide">guide</a>.</p>
    <img src="https://example.com/steps.png" alt="Steps" width="1024" height="768">
  </body>
</html>
HTML;

[$markdown, $metadata] = convert_with_metadata(
    $html,
    options: ['heading_style' => 'Atx'],
    metadataConfig: [
        'extract_document' => true,
        'extract_headers' => true,
        'extract_links' => true,
        'extract_images' => true,
    ]
);

echo "Title: " . $metadata['document']['title'] . "\n";
echo "Found " . count($metadata['links']) . " links\n";
```

#### Go Example

```go
package main

import (
	"encoding/json"
	"fmt"
	"log"

	"github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown"
)

func main() {
	html := `
	<html>
		<head>
			<title>Developer Guide</title>
			<meta name="description" content="Complete API reference">
		</head>
		<body>
			<h1>API Overview</h1>
			<p>Learn more at our <a href="https://api.example.com/docs">API docs</a>.</p>
			<img src="https://example.com/api-flow.png" alt="API Flow" width="1280" height="720">
		</body>
	</html>
	`

	markdown, metadata, err := htmltomarkdown.ConvertWithMetadata(html, &htmltomarkdown.MetadataConfig{
		ExtractDocument:     true,
		ExtractHeaders:      true,
		ExtractLinks:        true,
		ExtractImages:       true,
		ExtractStructuredData: false,
	})
	if err != nil {
		log.Fatal(err)
	}

	fmt.Println("Markdown:", markdown)
	fmt.Printf("Title: %s\n", metadata.Document.Title)
	fmt.Printf("Found %d links\n", len(metadata.Links))

	// Marshal to JSON if needed
	jsonBytes, _ := json.MarshalIndent(metadata, "", "  ")
	fmt.Println(string(jsonBytes))
}
```

#### Java Example

```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.ConversionResult;
import com.google.gson.Gson;
import com.google.gson.GsonBuilder;

public class MetadataExample {
    public static void main(String[] args) {
        String html = """
            <html>
              <head>
                <title>Java Guide</title>
                <meta name="description" content="Complete Java bindings documentation">
              </head>
              <body>
                <h1>Quick Start</h1>
                <p>Visit our <a href="https://github.com/Goldziher/html-to-markdown">GitHub</a>.</p>
                <img src="https://example.com/java-flow.png" alt="Flow diagram" width="1024" height="576">
              </body>
            </html>
            """;

        try {
            ConversionResult result = HtmlToMarkdown.convertWithMetadata(
                html,
                new HtmlToMarkdown.MetadataOptions()
                    .extractDocument(true)
                    .extractHeaders(true)
                    .extractLinks(true)
                    .extractImages(true)
            );

            System.out.println("Markdown:\n" + result.getMarkdown());
            System.out.println("Title: " + result.getMetadata().getDocument().getTitle());
            System.out.println("Links found: " + result.getMetadata().getLinks().size());

            // Pretty-print metadata as JSON
            Gson gson = new GsonBuilder().setPrettyPrinting().create();
            System.out.println(gson.toJson(result.getMetadata()));
        } catch (HtmlToMarkdown.ConversionException e) {
            System.err.println("Conversion failed: " + e.getMessage());
        }
    }
}
```

#### C# Example

```csharp
using HtmlToMarkdown;
using System.Text.Json;

var html = @"
<html>
  <head>
    <title>C# Guide</title>
    <meta name=""description"" content=""Official C# bindings documentation"">
  </head>
  <body>
    <h1>Introduction</h1>
    <p>See our <a href=""https://github.com/Goldziher/html-to-markdown"">repository</a>.</p>
    <img src=""https://example.com/csharp-arch.png"" alt=""Architecture"" width=""1200"" height=""675"">
  </body>
</html>
";

try
{
    var result = HtmlToMarkdownConverter.ConvertWithMetadata(
        html,
        new MetadataConfig
        {
            ExtractDocument = true,
            ExtractHeaders = true,
            ExtractLinks = true,
            ExtractImages = true,
        }
    );

    Console.WriteLine("Markdown:");
    Console.WriteLine(result.Markdown);

    Console.WriteLine($"Title: {result.Metadata.Document.Title}");
    Console.WriteLine($"Links found: {result.Metadata.Links.Count}");

    // Serialize metadata to JSON
    var options = new JsonSerializerOptions { WriteIndented = true };
    var json = JsonSerializer.Serialize(result.Metadata, options);
    Console.WriteLine(json);
}
catch (HtmlToMarkdownException ex)
{
    Console.Error.WriteLine($"Conversion failed: {ex.Message}");
}
```

See the individual binding READMEs for detailed metadata extraction options:

- **Python** – [Python README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/python/README.md)
- **TypeScript/Node.js** – [Node.js README](https://github.com/Goldziher/html-to-markdown/blob/main/crates/html-to-markdown-node/README.md) | [TypeScript README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/typescript/README.md)
- **Ruby** – [Ruby README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/ruby/README.md)
- **PHP** – [PHP README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/php/README.md)
- **Go** – [Go README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/go/README.md)
- **Java** – [Java README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/java/README.md)
- **C#/.NET** – [C# README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/csharp/README.md)
- **WebAssembly** – [WASM README](https://github.com/Goldziher/html-to-markdown/blob/main/crates/html-to-markdown-wasm/README.md)
- **Rust** – [Rust README](https://github.com/Goldziher/html-to-markdown/blob/main/crates/html-to-markdown/README.md)

### Python (v2 API)

```python
from html_to_markdown import convert, convert_with_inline_images, InlineImageConfig

html = "<h1>Hello</h1><p>Rust ❤️ Markdown</p>"
markdown = convert(html)

markdown, inline_images, warnings = convert_with_inline_images(
    '<img src="data:image/png;base64,...==" alt="Pixel">',
    image_config=InlineImageConfig(max_decoded_size_bytes=1024, infer_dimensions=True),
)
```

### Elixir

```elixir
{:ok, markdown} = HtmlToMarkdown.convert("<h1>Hello</h1>")

# Keyword options are supported (internally mapped to the Rust ConversionOptions struct)
HtmlToMarkdown.convert!("<p>Wrap me</p>", wrap: true, wrap_width: 32, preprocessing: %{enabled: true})
```

### Rust

```rust
use html_to_markdown_rs::{convert, ConversionOptions, HeadingStyle};

let html = "<h1>Welcome</h1><p>Fast conversion</p>";
let markdown = convert(html, None)?;

let options = ConversionOptions {
    heading_style: HeadingStyle::Atx,
    ..Default::default()
};
let markdown = convert(html, Some(options))?;
```

See the language-specific READMEs for complete configuration, hOCR workflows, and inline image extraction.

## Performance

Benchmarked on Apple M4 with complex real-world documents (Wikipedia articles, tables, lists):

### Operations per Second (higher is better)

Derived directly from `tools/runtime-bench/results/latest.json` (Apple M4, shared fixtures):

| Fixture                | Node.js (NAPI) | WASM | Python (PyO3) | Speedup (Node vs Python) |
| ---------------------- | -------------- | ---- | ------------- | ------------------------ |
| **Lists (Timeline)**   | 1,308          | 882  | 1,405         | **0.9×**                 |
| **Tables (Countries)** | 331            | 242  | 352           | **0.9×**                 |
| **Medium (Python)**    | 150            | 121  | 158           | **1.0×**                 |
| **Large (Rust)**       | 163            | 124  | 183           | **0.9×**                 |
| **Small (Intro)**      | 208            | 163  | 223           | **0.9×**                 |
| **HOCR German PDF**    | 2,944          | 1,637| 2,991         | **1.0×**                 |
| **HOCR Invoice**       | 27,326         | 7,775| 23,500        | **1.2×**                 |
| **HOCR Tables**        | 3,475          | 1,667| 3,464         | **1.0×**                 |

### Average Performance Summary

| Implementation        | Avg ops/sec (fixtures) | vs Python | Notes |
| --------------------- | ---------------------- | --------- | ----- |
| **Rust CLI/Binary**   | **4,996**              | **1.2× faster** | Preprocessing now stays in one pass + reuses `parse_owned`, so the CLI leads every fixture |
| **Node.js (NAPI-RS)** | **4,488**              | 1.0×      | Buffer/handle combo keeps Node within ~10 % of the Rust core while serving JS runtimes |
| **Ruby (magnus)**     | **4,278**              | 0.9×      | Still extremely fast; ~25 k ops/sec on HOCR invoices without extra work |
| **Python (PyO3)**     | **4,034**              | baseline  | Release-mode harness plus handle reuse keep it competitive, but it now trails Node/Rust |
| **WebAssembly**       | **1,576**              | 0.4×      | Portable option for Deno/browsers/edge using the new byte APIs |
| **PHP (ext)**         | **1,480**              | 0.4×      | Composer extension holds steady at 35–70 MB/s once the PIE build is installed |

### Key Insights

- **Rust now leads throughput**: the fused preprocessing + `parse_owned` pathway pushes the CLI to ~1.7 k ops/sec on the 129 KB lists page and ~31 k ops/sec on the HOCR invoice fixture.
- **Node.js trails by only a few percent** after the buffer/handle work—~1.3 k ops/sec on the lists fixture and 27 k ops/sec on HOCR invoices without any UTF-16 copies.
- **Python remains competitive** but now sits below Node/Rust (~4.0 k average ops/sec); stick to the v2 API to avoid the deprecated compatibility shim.
- **Elixir matches the Rust core** because the Rustler NIF executes the same `ConversionOptions` pipeline—benchmarks land between 170–1,460 ops/sec on the Wikipedia fixtures and >20 k ops/sec on micro HOCR payloads.
- **PHP and WASM stay in the 35–70 MB/s band**, which is plenty for Composer queues or edge runtimes as long as the extension/module is built ahead of time.
- **Rust CLI results now mirror the bindings**, since `task bench:bindings` runs the harness with `cargo run --release` by default—profile there, then push optimizations down into each FFI layer.

### Runtime Benchmarks (PHP / Ruby / Python / Node / WASM)

Measured on Apple M4 using the fixture-driven runtime harness in `tools/runtime-bench` (`task bench:bindings`). Every binding consumes the exact same HTML fixtures and hOCR samples from `test_documents/`:

| Document            | Size     | Ruby ops/sec | PHP ops/sec | Python ops/sec | Node ops/sec | WASM ops/sec | Elixir ops/sec | Rust ops/sec |
| ------------------- | -------- | ------------ | ----------- | -------------- | ------------ | ------------ | -------------- | ------------ |
| Lists (Timeline)    | 129 KB   | 1,349        | 533         | 1,405          | 1,308        | 882          | 1,463          | **1,700**    |
| Tables (Countries)  | 360 KB   | 326          | 118         | 352            | 331          | 242          | 357            | **416**      |
| Medium (Python)     | 657 KB   | 157          | 59          | 158            | 150          | 121          | 171            | **190**      |
| Large (Rust)        | 567 KB   | 174          | 65          | 183            | 163          | 124          | 174            | **220**      |
| Small (Intro)       | 463 KB   | 214          | 83          | 223            | 208          | 163          | 247            | **258**      |
| HOCR German PDF     | 44 KB    | 2,936        | 1,007       | **2,991**      | 2,944        | 1,637        | 3,113          | 2,760        |
| HOCR Invoice        | 4 KB     | 25,740       | 8,781       | 23,500         | 27,326       | 7,775        | 20,424         | **31,345**   |
| HOCR Embedded Tables| 37 KB    | 3,328        | 1,194       | 3,464          | **3,475**    | 1,667        | 3,366          | 3,080        |

The harness shells out to each runtime’s lightweight benchmark driver (`packages/*/bin/benchmark.*`, `crates/*/bin/benchmark.ts`), feeds fixtures defined in `tools/runtime-bench/fixtures/*.toml`, and writes machine-readable JSON reports (`tools/runtime-bench/results/latest.json`) for regression tracking. Add new languages or scenarios by extending those fixture files and drivers.

Use `task bench:bindings` to regenerate throughput numbers across all bindings or `task bench:bindings:profile` to capture CPU/memory samples while the benchmarks run. To focus on specific languages or fixtures (for example, `task bench:bindings -- --language elixir`), pass `--language` / `--fixture` directly to `cargo run --manifest-path tools/runtime-bench/Cargo.toml -- …`.

Need a call-stack view of the Rust core? Run `task flamegraph:rust` (or call the harness with `--language rust --flamegraph path.svg`) to profile a fixture and dump a ready-to-inspect flamegraph in `tools/runtime-bench/results/`.

**Note on Python performance**: The current Python bindings have optimization opportunities. The v2 API with direct `convert()` calls performs best; avoid the v1 compatibility layer for performance-critical applications.

## Compatibility (v1 → v2)

## Testing

Use the task runner to execute the entire matrix locally:

```bash
# All core test suites (Rust, Python, Ruby, Node, PHP, Go, C#, Elixir, Java)
task test

# Run the Wasmtime-backed WASM integration tests
task wasm:test:wasmtime
```

The Wasmtime suite builds the `html-to-markdown-wasm` artifact with the same flags used in CI and drives it through Wasmtime to ensure the non-JS runtime behaves exactly like the browser/Deno builds.

- V2’s Rust core sustains **150–210 MB/s** throughput; V1 averaged **≈ 2.5 MB/s** in its Python/BeautifulSoup implementation (60–80× faster).
- The Python package offers a compatibility shim in `html_to_markdown.v1_compat` (`convert_to_markdown`, `convert_to_markdown_stream`, `markdownify`). The shim is deprecated, emits `DeprecationWarning` on every call, and will be removed in v3.0—plan migrations now. Details and keyword mappings live in [Python README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/python/README.md#v1-compatibility).
- CLI flag changes, option renames, and other breaking updates are summarised in [CHANGELOG](https://github.com/Goldziher/html-to-markdown/blob/main/CHANGELOG.md#breaking-changes).

## Community

- Chat with us on [Discord](https://discord.gg/pXxagNK2zN)
- Explore the broader [Kreuzberg](https://kreuzberg.dev) document-processing ecosystem
- Sponsor development via [GitHub Sponsors](https://github.com/sponsors/Goldziher)
### Ruby

```ruby
require 'html_to_markdown'

html = '<h1>Hello</h1><p>Rust ❤️ Markdown</p>'
markdown = HtmlToMarkdown.convert(html, heading_style: :atx, wrap: true)

puts markdown
# # Hello
#
# Rust ❤️ Markdown
```

See the language-specific READMEs for complete configuration, hOCR workflows, and inline image extraction.
