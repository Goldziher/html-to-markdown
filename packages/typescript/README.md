# html-to-markdown (TypeScript)

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

High-performance HTML to Markdown converter for Node.js and Bun. This package wraps the
native `html-to-markdown-node` bindings and adds a TypeScript-friendly API plus a
cross-platform CLI.

```bash
# Native bindings (Node.js/Bun)
npm install html-to-markdown-node

# WebAssembly (browser/edge/Node without native toolchain)
npm install html-to-markdown-wasm
```

## Basic Usage

```ts
import { convert } from 'html-to-markdown-node';

const markdown = convert('<h1>Hello</h1>');
```

For WASM/bundler targets:

```ts
import { convert } from 'html-to-markdown-wasm';

const markdown = convert('<h1>Hello</h1>', null);
```

The package re-exports all conversion options exposed by the native bindings. See the
[core documentation](https://github.com/Goldziher/html-to-markdown) for complete
option descriptions.

### File Helpers

```ts
import { convertFile } from 'html-to-markdown-node';

const markdown = await convertFile('input.html');
```

### CLI

Use the Rust-native CLI for a packaged command-line interface:

```bash
cargo install html-to-markdown-cli
html-to-markdown --help
```

## Metadata Extraction API

The TypeScript/Node.js binding provides comprehensive metadata extraction capabilities through NAPI-RS, matching the Python binding's feature set.

The metadata extraction feature requires the `metadata` feature flag to be enabled at build time. When enabled, you can extract document metadata, headers, links, images, and structured data in a single conversion pass.

### API Functions

#### `convertWithMetadata(html, options?, metadataConfig?)`

Convert HTML to Markdown and extract comprehensive metadata.

```typescript
import { convertWithMetadata } from 'html-to-markdown';

const html = '<html lang="en"><head><title>My Article</title></head><body><h1>Hello</h1></body></html>';
const { markdown, metadata } = convertWithMetadata(html, undefined, {
  extractHeaders: true,
  extractLinks: true,
  extractImages: true,
  extractStructuredData: true,
});

console.log(metadata.document.title);     // "My Article"
console.log(metadata.document.language);  // "en"
console.log(metadata.headers);            // [{ level: 1, text: "Hello", ... }]
```

**Parameters:**
- `html` (string): HTML content to convert
- `options?` (JsConversionOptions): Optional conversion configuration
- `metadataConfig?` (JsMetadataConfig): Optional metadata extraction configuration

**Returns:** `JsMetadataExtraction`

#### `convertWithMetadataBuffer(html, options?, metadataConfig?)`

Convert HTML from Buffer/Uint8Array to Markdown with metadata extraction.

Avoids creating intermediate JavaScript strings by accepting raw bytes. Auto-detects UTF-8 encoding.

```typescript
import { convertWithMetadataBuffer } from 'html-to-markdown';

const buffer = Buffer.from('<h1>Title</h1>');
const { markdown, metadata } = convertWithMetadataBuffer(buffer);
```

#### `convertFileWithMetadata(filePath, options?, metadataConfig?)`

Asynchronously convert an HTML file to Markdown with metadata extraction.

```typescript
import { convertFileWithMetadata } from 'html-to-markdown';

const { markdown, metadata } = await convertFileWithMetadata('page.html');
console.log(metadata.document.title);
```

#### `convertStreamWithMetadata(stream, options?, metadataConfig?)`

Convert HTML from a readable stream (stdin, file stream, etc.) with metadata extraction.

```typescript
import { convertStreamWithMetadata } from 'html-to-markdown';
import fs from 'node:fs';

const stream = fs.createReadStream('large-page.html', 'utf8');
const { markdown, metadata } = await convertStreamWithMetadata(stream);
```

### Configuration Types

#### `JsMetadataConfig`

```typescript
interface JsMetadataConfig {
  extractHeaders?: boolean;              // Extract h1-h6 elements (default: true)
  extractLinks?: boolean;                // Extract <a> elements (default: true)
  extractImages?: boolean;               // Extract <img> and inline SVG (default: true)
  extractStructuredData?: boolean;       // Extract JSON-LD, Microdata, RDFa (default: true)
  maxStructuredDataSize?: number;        // Max bytes for structured data (default: 1,000,000)
}
```

**Example:**
```typescript
const config: JsMetadataConfig = {
  extractHeaders: true,
  extractLinks: true,
  extractImages: false,  // Skip image extraction
  extractStructuredData: true,
  maxStructuredDataSize: 500_000,  // 500KB limit
};
```

### Result Types

#### `JsMetadataExtraction`

The complete result of metadata extraction.

```typescript
interface JsMetadataExtraction {
  markdown: string;                    // Converted markdown output
  metadata: JsExtendedMetadata;        // Extracted metadata
}
```

#### `JsExtendedMetadata`

Container for all metadata categories.

```typescript
interface JsExtendedMetadata {
  document: JsDocumentMetadata;        // Document-level metadata
  headers: JsHeaderMetadata[];         // H1-H6 elements with hierarchy
  links: JsLinkMetadata[];             // Extracted hyperlinks
  images: JsImageMetadata[];           // Extracted images
  structuredData: JsStructuredData[];  // JSON-LD, Microdata, RDFa blocks
}
```

#### `JsDocumentMetadata`

Document-level metadata from the `<head>` section.

```typescript
interface JsDocumentMetadata {
  title?: string;                      // <title> tag content
  description?: string;                // meta[name="description"]
  keywords: string[];                  // meta[name="keywords"], split on commas
  author?: string;                     // meta[name="author"]
  canonicalUrl?: string;               // link[rel="canonical"]
  baseHref?: string;                   // <base href="">
  language?: string;                   // lang attribute
  textDirection?: string;              // "ltr" | "rtl" | "auto"
  openGraph: Record<string, string>;   // og:* properties
  twitterCard: Record<string, string>; // twitter:* properties
  metaTags: Record<string, string>;    // Other meta tags
}
```

**Example:**
```typescript
const metadata = result.metadata.document;

if (metadata.title) {
  console.log(`Article: ${metadata.title}`);
}

if (metadata.openGraph.image) {
  console.log(`Social preview: ${metadata.openGraph.image}`);
}

// Check language and directionality for RTL support
if (metadata.textDirection === 'rtl') {
  // Handle Arabic, Hebrew, etc.
}
```

#### `JsHeaderMetadata`

Header element (h1-h6) with hierarchy information.

```typescript
interface JsHeaderMetadata {
  level: number;        // 1-6
  text: string;         // Normalized text content
  id?: string;          // HTML id attribute
  depth: number;        // Nesting depth in document tree
  htmlOffset: number;   // Byte offset in original HTML
}
```

**Example:**
```typescript
const toc = result.metadata.headers.map(h => ({
  level: h.level,
  title: h.text,
  anchor: h.id || h.text.toLowerCase().replace(/\s+/g, '-'),
}));
```

#### `JsLinkMetadata`

Extracted hyperlink with classification.

```typescript
interface JsLinkMetadata {
  href: string;                        // The URL
  text: string;                        // Link text
  title?: string;                      // title attribute
  linkType: string;                    // Classification (see below)
  rel: string[];                       // rel attribute values
  attributes: Record<string, string>;  // Other HTML attributes
}
```

**Link Types:**
- `"anchor"` - Fragment/anchor link (`#section`)
- `"internal"` - Same-domain link (`/page`, `../relative`, `./current`)
- `"external"` - Different domain (`https://other.com`)
- `"email"` - Email link (`mailto:`)
- `"phone"` - Phone link (`tel:`)
- `"other"` - Other protocol or unclassifiable

**Example:**
```typescript
const externalLinks = result.metadata.links.filter(l => l.linkType === 'external');
const emailLinks = result.metadata.links.filter(l => l.linkType === 'email');

for (const link of externalLinks) {
  console.log(`${link.text}: ${link.href}`);
}
```

#### `JsImageMetadata`

Extracted image with source classification.

```typescript
interface JsImageMetadata {
  src: string;                         // Image source (URL or data URI)
  alt?: string;                        // Alternative text
  title?: string;                      // title attribute
  dimensions?: [number, number];       // [width, height] if available
  imageType: string;                   // Classification (see below)
  attributes: Record<string, string>;  // Other HTML attributes
}
```

**Image Types:**
- `"data_uri"` - Base64 or other encoded data URI
- `"inline_svg"` - Inline `<svg>` element
- `"external"` - External URL (`http://`, `https://`)
- `"relative"` - Relative path (no protocol)

**Example:**
```typescript
// Extract downloadable images
const downloadableImages = result.metadata.images.filter(
  img => img.imageType === 'external' && img.src.startsWith('https://')
);

// Find images without alt text (accessibility issue)
const missingAlt = result.metadata.images.filter(img => !img.alt);
```

#### `JsStructuredData`

Machine-readable structured data block.

```typescript
interface JsStructuredData {
  dataType: string;       // "json_ld" | "microdata" | "rdfa"
  rawJson: string;        // JSON string (for JSON-LD)
  schemaType?: string;    // Detected schema type (e.g., "Article")
}
```

**Example:**
```typescript
const jsonLdBlocks = result.metadata.structuredData.filter(
  d => d.dataType === 'json_ld'
);

for (const block of jsonLdBlocks) {
  const schema = JSON.parse(block.rawJson);
  console.log(`Found ${block.schemaType}: ${schema.name}`);
}
```

### Use Cases

#### Extract Article Metadata

```typescript
import { convertWithMetadata } from 'html-to-markdown';

function extractArticleMetadata(html: string) {
  const { markdown, metadata } = convertWithMetadata(html, { headingStyle: 'atx' });

  return {
    title: metadata.document.title,
    description: metadata.document.description,
    author: metadata.document.author,
    markdown,
    toc: metadata.headers.map(h => ({
      level: h.level,
      text: h.text,
    })),
  };
}
```

#### Build Table of Contents

```typescript
function buildTableOfContents(html: string) {
  const { metadata } = convertWithMetadata(html);

  return metadata.headers.map(h => ({
    level: h.level,
    text: h.text,
    anchor: h.id || slugify(h.text),
    depth: h.depth,
  }));
}
```

#### Validate Links

```typescript
function findBrokenLinks(html: string) {
  const { metadata } = convertWithMetadata(html);

  return {
    anchors: metadata.links.filter(l => l.linkType === 'anchor'),
    internalLinks: metadata.links.filter(l => l.linkType === 'internal'),
    externalLinks: metadata.links.filter(l => l.linkType === 'external'),
    emailLinks: metadata.links.filter(l => l.linkType === 'email'),
  };
}
```

#### Extract SEO Metadata

```typescript
function extractSeoMetadata(html: string) {
  const { metadata } = convertWithMetadata(html);
  const doc = metadata.document;

  return {
    title: doc.title,
    metaDescription: doc.description,
    keywords: doc.keywords,
    canonical: doc.canonicalUrl,
    language: doc.language,
    openGraph: {
      title: doc.openGraph.title,
      description: doc.openGraph.description,
      image: doc.openGraph.image,
      url: doc.openGraph.url,
    },
    twitter: {
      card: doc.twitterCard.card,
      creator: doc.twitterCard.creator,
      site: doc.twitterCard.site,
    },
  };
}
```

#### Check Accessibility Issues

```typescript
function checkAccessibility(html: string) {
  const { metadata } = convertWithMetadata(html);

  const issues = {
    imagesWithoutAlt: metadata.images.filter(img => !img.alt),
    linksWithoutText: metadata.links.filter(link => !link.text.trim()),
    missingAltText: metadata.images
      .filter(img => !img.alt)
      .map(img => `${img.src}: missing alt text`),
  };

  return issues;
}
```

### Performance Considerations

1. **Single-Pass Collection**: Metadata extraction happens during the HTML parsing and tree traversal, with zero overhead when disabled.

2. **Memory-Efficient**: Collections are pre-allocated based on typical document sizes (32 headers, 64 links, 16 images).

3. **Structured Data Size Limits**: Large JSON-LD blocks are skipped if they exceed the configured limit to prevent memory exhaustion.

4. **Buffer Processing**: Use `convertWithMetadataBuffer()` to avoid creating intermediate JavaScript strings for large files.

5. **Streaming**: Use `convertStreamWithMetadata()` for processing large files from disk or network.

### Feature Detection

The metadata feature is optional. Applications should gracefully degrade:

```typescript
import { hasMetadataSupport } from 'html-to-markdown';

if (hasMetadataSupport()) {
  // Metadata extraction is available
  const { markdown, metadata } = convertWithMetadata(html);
} else {
  // Fallback to basic conversion
  const markdown = convert(html);
}
```

### Building with Metadata Feature

To enable metadata extraction in the native build:

```bash
# Build Rust native module with metadata feature
cargo build -p html-to-markdown-node --features metadata --release

# Build TypeScript wrapper
cd packages/typescript
pnpm build
```

Or using the Taskfile:

```bash
task build:metadata
```

### Feature Availability

The metadata extraction feature is **enabled by default** in the published npm package. If building from source, the feature is enabled via the `default` feature flag in Cargo.toml.

You can check if metadata support is available at runtime using the `hasMetadataSupport()` function:

```typescript
import { hasMetadataSupport, convertWithMetadata, convert } from 'html-to-markdown';

if (hasMetadataSupport()) {
  // Metadata extraction is available
  const { markdown, metadata } = convertWithMetadata(html);
} else {
  // Fallback to basic conversion
  const markdown = convert(html);
}
```

### Testing Metadata Extraction

The metadata extraction is thoroughly tested in `tests/metadata.spec.ts`:

```bash
# Run metadata tests
pnpm --filter html-to-markdown test -- metadata.spec.ts
```

Test coverage includes:
- Document metadata extraction
- Header hierarchy tracking
- Link classification
- Image type detection
- Structured data handling
- Configuration flags
- Special characters and encoding
- Size limits for structured data

## Performance (Apple M4)

This package wraps the native `html-to-markdown-node` bindings, so throughput matches the Node README. Benchmarks come from `task bench:bindings -- --language node` and use shared Wikipedia + hOCR fixtures:

| Document               | Size   | ops/sec |
| ---------------------- | ------ | ------- |
| Lists (Timeline)       | 129 KB | 1,308   |
| Tables (Countries)     | 360 KB | 331     |
| Medium (Python)        | 657 KB | 150     |
| Large (Rust)           | 567 KB | 163     |
| Small (Intro)          | 463 KB | 208     |
| hOCR German PDF        | 44 KB  | 2,944   |
| hOCR Invoice           | 4 KB   | 27,326  |
| hOCR Embedded Tables   | 37 KB  | 3,475   |

> Run `task bench:bindings -- --language node` to regenerate the data locally.
