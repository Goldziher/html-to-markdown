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

## Error Handling

Conversion errors raise `Error` with a Rust-provided message. Inputs that look like binary data (e.g., PDF bytes
coerced to a string) are rejected with an `Invalid input` message.

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

## Visitor Pattern

The visitor pattern allows you to customize HTML→Markdown conversion by providing callbacks for specific HTML elements. This is useful for implementing domain-specific transformations, element filtering, custom formatting rules, and analytics during conversion. TypeScript visitors support **both synchronous and asynchronous methods** with seamless async/await integration.

### Overview

Visitors intercept HTML element conversion, allowing you to:
- **Filter content**: Skip certain elements (ads, tracking pixels, unwanted sections)
- **Transform output**: Replace default Markdown with custom formatting (footnotes, admonitions, etc.)
- **Validate**: Enforce constraints during conversion (e.g., alt text on images, link validation)
- **Enrich**: Add metadata or side effects during conversion (analytics, link tracking)
- **Preserve HTML**: Keep unsupported elements as raw HTML instead of attempting conversion

The visitor pattern is essential for domain-specific markdown dialects, custom markdown rendering, and content processing pipelines where you need fine-grained control over specific elements.

### Basic Example

```typescript
import { convertWithVisitor } from 'html-to-markdown-node';

interface MyVisitor {
  visitLink?(ctx: NodeContext, href: string, text: string, title?: string): VisitResult;
  visitImage?(ctx: NodeContext, src: string, alt?: string, title?: string): VisitResult;
}

const visitor: MyVisitor = {
  visitLink: (ctx, href, text, title) => {
    // Customize link conversion
    return { type: 'custom', output: `[${text}](${href})` };
  },
  visitImage: (ctx, src, alt, title) => {
    // Skip all images
    return { type: 'skip' };
  },
};

const html = '<a href="/page">Link</a><img src="pic.jpg" />';
const markdown = convertWithVisitor(html, { visitor });
// Output: [Link](/page)
// Images are removed from output
```

### NodeContext: Element Metadata

Every visitor method receives a `ctx` parameter (NodeContext) with comprehensive information about the current element:

```typescript
interface NodeContext {
  nodeType: string;        // "text" | "element" | "heading" | "link" | etc.
  tagName: string;         // Raw HTML tag name ("a", "img", "div", etc.)
  attributes: Record<string, string>; // All HTML attributes as key-value pairs
  depth: number;           // Nesting depth in DOM tree (0 = root)
  indexInParent: number;   // Zero-based index among siblings
  parentTag: string | null; // Parent element's tag, or null if root
  isInline: boolean;       // Whether treated as inline vs block element
}
```

Use context information to make intelligent decisions:

```typescript
interface SmartVisitor {
  visitLink?(ctx: NodeContext, href: string, text: string, title?: string): VisitResult;
}

const visitor: SmartVisitor = {
  visitLink: (ctx, href, text, title) => {
    // Only rewrite external links
    if (href.startsWith('http')) {
      return {
        type: 'custom',
        output: `[${text}](${href}){.external}`,
      };
    }
    // Keep internal links as-is
    return { type: 'continue' };
  },
};
```

### VisitResult Types

Return a visitor result from visitor methods to control conversion behavior:

**Continue with default conversion:**
```typescript
visitLink: (ctx, href, text, title) => {
  return { type: 'continue' };
}
```

**Use custom markdown output:**
```typescript
visitLink: (ctx, href, text, title) => {
  return { type: 'custom', output: `[${text}](${href})` };
}
```

**Skip element entirely:**
```typescript
visitImage: (ctx, src, alt, title) => {
  return { type: 'skip' };  // Image is not rendered
}
```

**Preserve original HTML:**
```typescript
visitVideo: (ctx, src) => {
  return { type: 'preserveHtml' };  // Keep as `<video src="..."></video>`
}
```

**Stop conversion with error:**
```typescript
visitLink: (ctx, href, text, title) => {
  if (!href.startsWith(('http://', 'https://', '/'))) {
    return { type: 'error', message: `Invalid URL: ${href}` };
  }
  return { type: 'continue' };
}
```

### Visitor Methods Reference

The visitor pattern supports the following 41 methods. Implement only the ones relevant to your use case:

**Text & Inline Elements:**
- `visitText(ctx, text)` – Plain text nodes
- `visitStrong(ctx, text)` – Bold text (`<strong>`, `<b>`)
- `visitEmphasis(ctx, text)` – Italic text (`<em>`, `<i>`)
- `visitStrikethrough(ctx, text)` – Strikethrough (`<s>`, `<del>`)
- `visitUnderline(ctx, text)` – Underlined text (`<u>`)
- `visitSubscript(ctx, text)` – Subscript (`<sub>`)
- `visitSuperscript(ctx, text)` – Superscript (`<sup>`)
- `visitMark(ctx, text)` – Highlighted/marked text (`<mark>`)
- `visitCodeInline(ctx, code)` – Inline code (`<code>`)
- `visitLineBreak(ctx)` – Line breaks (`<br>`)

**Links & Media:**
- `visitLink(ctx, href, text, title?)` – Hyperlinks (`<a>`)
- `visitImage(ctx, src, alt?, title?)` – Images (`<img>`)
- `visitAudio(ctx, src)` – Audio files (`<audio>`)
- `visitVideo(ctx, src)` – Video files (`<video>`)
- `visitIframe(ctx, src)` – Embedded content (`<iframe>`)

**Block Elements:**
- `visitElementStart(ctx)` – Element opening (all elements)
- `visitElementEnd(ctx, output)` – Element closing (all elements)
- `visitHeading(ctx, level, text, id?)` – Headers (`<h1>`–`<h6>`)
- `visitCodeBlock(ctx, lang?, code)` – Code blocks (`<pre><code>`)
- `visitBlockquote(ctx, content, depth)` – Block quotes (`<blockquote>`)
- `visitHorizontalRule(ctx)` – Horizontal rule (`<hr>`)

**Lists:**
- `visitListStart(ctx, ordered)` – List start (`<ul>`, `<ol>`)
- `visitListItem(ctx, ordered, marker, text)` – List item (`<li>`)
- `visitListEnd(ctx, ordered, output)` – List end

**Tables:**
- `visitTableStart(ctx)` – Table start (`<table>`)
- `visitTableRow(ctx, cells, isHeader)` – Table row (`<tr>`)
- `visitTableEnd(ctx, output)` – Table end

**Advanced Elements:**
- `visitDefinitionListStart(ctx)` – Definition list start (`<dl>`)
- `visitDefinitionTerm(ctx, text)` – Definition term (`<dt>`)
- `visitDefinitionDescription(ctx, text)` – Definition description (`<dd>`)
- `visitDefinitionListEnd(ctx, output)` – Definition list end
- `visitFigureStart(ctx)` – Figure start (`<figure>`)
- `visitFigcaption(ctx, text)` – Figure caption (`<figcaption>`)
- `visitFigureEnd(ctx, output)` – Figure end
- `visitDetails(ctx, open)` – Collapsible details (`<details>`)
- `visitSummary(ctx, text)` – Summary elements (`<summary>`)

**Forms:**
- `visitForm(ctx, action, method)` – Form element (`<form>`)
- `visitInput(ctx, inputType, name, value)` – Input field (`<input>`)
- `visitButton(ctx, text)` – Button (`<button>`)

**Custom:**
- `visitCustomElement(ctx, tagName, html)` – Unknown/custom elements

### Practical Examples

#### Example 1: Custom Image Handling

Rewrite images with CDN optimization:

```typescript
import { convertWithVisitor, NodeContext, VisitResult } from 'html-to-markdown-node';

interface ImageOptimizer {
  cdnUrl: string;
  visitImage?(ctx: NodeContext, src: string, alt?: string, title?: string): VisitResult;
}

const optimizer: ImageOptimizer = {
  cdnUrl: 'https://cdn.example.com/image',
  visitImage: function(ctx, src, alt, title) {
    // Rewrite URLs to use CDN
    const optimizedSrc = `${this.cdnUrl}?src=${encodeURIComponent(src)}`;
    return {
      type: 'custom',
      output: `![${alt || ''}](${optimizedSrc} "${title || alt || ''}")`,
    };
  },
};

const html = '<p><img src="/images/logo.png" alt="Logo" /></p>';
const markdown = convertWithVisitor(html, { visitor: optimizer });
// Output: ![Logo](https://cdn.example.com/image?src=%2Fimages%2Flogo.png "Logo")
```

#### Example 2: Content Filtering

Filter out ads, tracking pixels, and external links:

```typescript
interface ContentFilter {
  visitLink?(ctx: NodeContext, href: string, text: string, title?: string): VisitResult;
  visitImage?(ctx: NodeContext, src: string, alt?: string, title?: string): VisitResult;
  visitIframe?(ctx: NodeContext, src: string): VisitResult;
}

const filter: ContentFilter = {
  visitLink: (ctx, href, text, title) => {
    // Only allow internal links
    if (href.startsWith('/') || href.startsWith('https://example.com')) {
      return { type: 'continue' };
    }
    return { type: 'skip' };
  },
  visitImage: (ctx, src, alt, title) => {
    // Skip tracking pixels and ads
    if (src.includes('tracking') || src.toLowerCase().includes('ad')) {
      return { type: 'skip' };
    }
    // Skip tiny images (likely tracking pixels)
    const width = parseInt(ctx.attributes.width || '0', 10);
    if (width > 0 && width < 10) {
      return { type: 'skip' };
    }
    return { type: 'continue' };
  },
  visitIframe: (ctx, src) => {
    // Block iframes entirely
    return { type: 'skip' };
  },
};

const html = `
  <p>Read more: <a href="https://ads.com">Click here</a></p>
  <p><a href="/about">About us</a></p>
  <img src="/tracking.gif" alt="tracker" width="1" />
  <img src="/logo.png" alt="Logo" />
  <iframe src="https://evil.com/tracker"></iframe>
`;

const markdown = convertWithVisitor(html, { visitor: filter });
// External links, tracking pixels, and iframes are removed
```

#### Example 3: Link Footnote References

Convert external links to footnote-style references:

```typescript
interface LinkFootnoteFormatter {
  links: Array<{
    index: number;
    href: string;
    text: string;
    title?: string;
  }>;
  linkIndex: number;
  visitLink?(ctx: NodeContext, href: string, text: string, title?: string): VisitResult;
  footnotesSection(): string;
}

const formatter: LinkFootnoteFormatter = {
  links: [],
  linkIndex: 0,
  visitLink(ctx, href, text, title) {
    this.linkIndex++;
    this.links.push({
      index: this.linkIndex,
      href,
      text,
      title,
    });
    // Return link text with superscript index
    return {
      type: 'custom',
      output: `${text}[^${this.linkIndex}]`,
    };
  },
  footnotesSection() {
    if (this.links.length === 0) return '';

    const lines = ['\n\n---\n\n'];
    for (const link of this.links) {
      lines.push(`[^${link.index}]: ${link.href}`);
      if (link.title) {
        lines.push(` "${link.title}"`);
      }
      lines.push('\n');
    }
    return lines.join('');
  },
};

const html = `
  <p>Visit <a href="https://example.com" title="Example Site">Example</a>
  and <a href="https://rust-lang.org">Rust</a> for more info.</p>
`;

const markdown = convertWithVisitor(html, { visitor: formatter });
const fullMarkdown = markdown + formatter.footnotesSection();

// Output:
// Visit Example[^1] and Rust[^2] for more info.
//
// ---
//
// [^1]: https://example.com "Example Site"
// [^2]: https://rust-lang.org
```

#### Example 4: Analytics and Content Extraction

Track link types and image usage during conversion:

```typescript
interface ContentAnalytics {
  stats: {
    internal: number;
    external: number;
    email: number;
    images: number;
    totalLinks: string[];
  };
  visitLink?(ctx: NodeContext, href: string, text: string, title?: string): VisitResult;
  visitImage?(ctx: NodeContext, src: string, alt?: string, title?: string): VisitResult;
}

const analytics: ContentAnalytics = {
  stats: {
    internal: 0,
    external: 0,
    email: 0,
    images: 0,
    totalLinks: [],
  },
  visitLink(ctx, href, text, title) {
    this.stats.totalLinks.push(href);

    if (href.startsWith('mailto:')) {
      this.stats.email++;
    } else if (href.startsWith('http')) {
      this.stats.external++;
      // Add tracking parameter to external links
      const separator = href.includes('?') ? '&' : '?';
      const tracked = `${href}${separator}utm_source=markdown`;
      return {
        type: 'custom',
        output: `[${text}](${tracked})`,
      };
    } else {
      this.stats.internal++;
    }
    return { type: 'continue' };
  },
  visitImage(ctx, src, alt, title) {
    this.stats.images++;
    return { type: 'continue' };
  },
};

const html = `
  <a href="/home">Home</a>
  <a href="https://example.com">External</a>
  <a href="mailto:test@example.com">Email</a>
  <img src="/logo.png" alt="Logo" />
`;

const markdown = convertWithVisitor(html, { visitor: analytics });
console.log('Link stats:', analytics.stats);
// => { internal: 1, external: 1, email: 1, images: 1, totalLinks: [...] }
```

#### Example 5: Asynchronous Visitor with URL Validation

TypeScript visitors support async methods for I/O operations:

```typescript
interface AsyncLinkValidator {
  visitLink?(ctx: NodeContext, href: string, text: string, title?: string): VisitResult | Promise<VisitResult>;
  checkUrl(url: string): Promise<boolean>;
}

const validator: AsyncLinkValidator = {
  async visitLink(ctx, href, text, title) {
    if (href.startsWith('http')) {
      try {
        const isValid = await this.checkUrl(href);
        if (!isValid) {
          return {
            type: 'custom',
            output: `[${text}](${href}) [dead]`,
          };
        }
      } catch (e) {
        // Silently continue on error
      }
    }
    return { type: 'continue' };
  },
  async checkUrl(url: string): Promise<boolean> {
    try {
      const response = await fetch(url, { method: 'HEAD', timeout: 5000 });
      return response.ok;
    } catch {
      return false;
    }
  },
};

const html = '<p><a href="https://example.com">Valid</a> <a href="https://dead.invalid">Dead</a></p>';
const markdown = await convertWithVisitor(html, { visitor: validator });
// Asynchronous validation happens during conversion
```

### Integration with ConversionOptions

Visitors work seamlessly with all conversion options:

```typescript
import { convertWithVisitor, ConversionOptions } from 'html-to-markdown-node';

interface MyVisitor {
  visitLink?(ctx: NodeContext, href: string, text: string, title?: string): VisitResult;
}

const visitor: MyVisitor = {
  visitLink: (ctx, href, text) => {
    return { type: 'custom', output: `[${text}](${href})` };
  },
};

const options: ConversionOptions = {
  headingStyle: 'atx',
  listIndentWidth: 2,
  wrap: true,
  wrapWidth: 80,
};

const markdown = convertWithVisitor(html, { options, visitor });
```

### Performance Considerations

1. **Minimal overhead**: Visitor methods are only called for elements where you define callbacks; elements without visitor methods use fast default conversion.

2. **Async methods**: Use async visitor methods for I/O operations (URL validation, API calls, database lookups) without blocking the main thread.

3. **Stateful visitors**: It's safe to collect data across elements (see analytics and footnote examples) because visitor methods execute in order during tree traversal.

4. **Memory efficiency**: Avoid storing large intermediate results in visitor state; process and discard data as conversion progresses.

5. **Error handling**: Return `{ type: 'error', message }` to fail fast and halt conversion; don't throw exceptions from visitor methods.

## Performance (Apple M4)

This package wraps the native `html-to-markdown-node` bindings, so throughput matches the Node README. Benchmarks come from the shared fixture harness in `tools/benchmark-harness`:

| Document               | Size   | ops/sec |
| ---------------------- | ------ | ------- |
| Lists (Timeline)       | 129 KB | 3,137   |
| Tables (Countries)     | 360 KB | 932     |
| Medium (Python)        | 657 KB | 460     |
| Large (Rust)           | 567 KB | 554     |
| Small (Intro)          | 463 KB | 627     |
| hOCR German PDF        | 44 KB  | 8,724   |
| hOCR Invoice           | 4 KB   | 96,138  |
| hOCR Embedded Tables   | 37 KB  | 9,591   |

> Run `task bench:harness -- --frameworks node` to regenerate the data locally.
