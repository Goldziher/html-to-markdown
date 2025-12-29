# html-to-markdown (TypeScript)

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg?logo=rust&label=crates.io)](https://crates.io/crates/html-to-markdown-rs)
[![npm (node)](https://img.shields.io/npm/v/%40kreuzberg%2Fhtml-to-markdown-node.svg?logo=npm)](https://www.npmjs.com/package/@kreuzberg/html-to-markdown-node)
[![npm (wasm)](https://img.shields.io/npm/v/%40kreuzberg%2Fhtml-to-markdown-wasm.svg?logo=npm)](https://www.npmjs.com/package/@kreuzberg/html-to-markdown-wasm)
[![PyPI](https://img.shields.io/pypi/v/html-to-markdown.svg?logo=pypi)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/Goldziher.HtmlToMarkdown.svg)](https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown.svg)](https://pkg.go.dev/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

High-performance HTML to Markdown converter for Node.js and Bun with full TypeScript support. This package wraps native `@kreuzberg/html-to-markdown-node` bindings and provides a type-safe API.

## Installation

```bash
# Native bindings (Node.js/Bun) - Recommended
npm install @kreuzberg/html-to-markdown
pnpm add @kreuzberg/html-to-markdown
yarn add @kreuzberg/html-to-markdown
bun add @kreuzberg/html-to-markdown

# WebAssembly (browser/edge/Node without native toolchain)
npm install @kreuzberg/html-to-markdown-wasm
```

## Migration Guide (v2.18.x → v2.19.0)

### Breaking Change: Scoped npm Packages

In v2.19.0, npm packages were moved to the `@kreuzberg` scope to align with the Kreuzberg.dev organization.

#### Package Installation Update

**Before (v2.18.x):**
```bash
npm install html-to-markdown-node
npm install html-to-markdown-wasm
```

**After (v2.19.0+):**
```bash
npm install @kreuzberg/html-to-markdown-node
npm install @kreuzberg/html-to-markdown-wasm
```

#### Import Statement Update

**Before:**
```typescript
import { convert } from 'html-to-markdown-node';
import { convert } from 'html-to-markdown-wasm';
```

**After:**
```typescript
import { convert } from '@kreuzberg/html-to-markdown-node';
import { convert } from '@kreuzberg/html-to-markdown-wasm';
```

#### TypeScript Declaration Update

Update your TypeScript configuration if you have imports from the old package name:

**Before (tsconfig.json or import aliases):**
```json
{
  "compilerOptions": {
    "paths": {
      "html-to-markdown": ["node_modules/html-to-markdown-node"]
    }
  }
}
```

**After:**
```json
{
  "compilerOptions": {
    "paths": {
      "@kreuzberg/html-to-markdown": ["node_modules/@kreuzberg/html-to-markdown-node"]
    }
  }
}
```

#### Deno Update

**Before:**
```typescript
import { convert } from "npm:html-to-markdown-wasm";
```

**After:**
```typescript
import { convert } from "npm:@kreuzberg/html-to-markdown-wasm";
```

#### Summary of Changes

- All npm packages now use `@kreuzberg` scope
- `html-to-markdown-node` → `@kreuzberg/html-to-markdown-node`
- `html-to-markdown-wasm` → `@kreuzberg/html-to-markdown-wasm`
- TypeScript types and APIs are identical
- No functional changes to the library

## Quick Start

**Basic conversion with type safety:**
```typescript
import { convert } from '@kreuzberg/html-to-markdown';

const markdown: string = convert('<h1>Hello World</h1>');
console.log(markdown); // # Hello World
```

**With conversion options:**
```typescript
import { convert, ConversionOptions } from '@kreuzberg/html-to-markdown';

const options: ConversionOptions = {
  headingStyle: 'atx',
  listIndentWidth: 2,
  wrap: true,
};

const markdown = convert('<h1>Title</h1><p>Content</p>', options);
```

**TypeScript interfaces for type safety:**
```typescript
interface ConversionOptions {
  headingStyle?: 'atx' | 'setext';
  listIndentWidth?: number;
  wrap?: boolean;
  wrapWidth?: number;
  // ... more options
}
```

**File and stream helpers:**
```typescript
import { convertFile, convertBuffer } from '@kreuzberg/html-to-markdown';

// From file
const markdown = await convertFile('page.html');

// From Buffer/Uint8Array
const buffer = Buffer.from('<h1>Title</h1>');
const markdown = convertBuffer(buffer);
```

## API Reference

### Core Functions

#### `convert(html: string, options?: ConversionOptions): string`
Convert HTML string to Markdown.

#### `convertBuffer(buffer: Buffer | Uint8Array, options?: ConversionOptions): string`
Convert HTML from Buffer/Uint8Array (avoids string allocation overhead).

#### `convertFile(filePath: string, options?: ConversionOptions): Promise<string>`
Asynchronously convert an HTML file to Markdown.

#### `convertStream(stream: NodeJS.ReadableStream, options?: ConversionOptions): Promise<string>`
Convert HTML from a readable stream (stdin, file stream, network).

### Metadata Extraction Functions

Requires `metadata` feature flag.

#### `convertWithMetadata(html: string, options?, metadataConfig?): { markdown: string; metadata: JsExtendedMetadata }`
Convert and extract document metadata, headers, links, images, and structured data.

#### `convertWithMetadataBuffer(buffer: Buffer | Uint8Array, options?, metadataConfig?): JsMetadataExtraction`
Convert from Buffer with metadata extraction.

#### `convertFileWithMetadata(filePath: string, options?, metadataConfig?): Promise<JsMetadataExtraction>`
Convert HTML file with metadata extraction.

#### `convertStreamWithMetadata(stream: NodeJS.ReadableStream, options?, metadataConfig?): Promise<JsMetadataExtraction>`
Convert stream with metadata extraction.

#### `hasMetadataSupport(): boolean`
Check if metadata extraction is available at runtime.

### Visitor Pattern Functions

Custom element callbacks for fine-grained conversion control.

#### `convertWithVisitor(html: string, config: { visitor: Visitor; options?: ConversionOptions }): string | Promise<string>`
Convert with visitor callbacks for element interception.

#### `convertWithAsyncVisitor(html: string, config: { visitor: AsyncVisitor; options?: ConversionOptions }): Promise<string>`
Convert with async visitor methods for I/O operations.

## Type Definitions

### ConversionOptions
```typescript
interface ConversionOptions {
  headingStyle?: 'atx' | 'setext';           // # Style or underline style
  bulletListMarker?: '-' | '*' | '+';        // List marker
  codeBlockStyle?: 'fenced' | 'indented';    // Code block format
  horizontalRule?: string;                   // --- or *** or ___
  listIndentWidth?: number;                  // Indentation (default: 4)
  wrap?: boolean;                            // Enable text wrapping
  wrapWidth?: number;                        // Wrap column width
  preserveNotices?: boolean;                 // Keep HTML comments
  sanitize?: boolean;                        // Remove unsafe HTML (default: true)
  headingPrefix?: string;                    // Prefix for headings
  strongDelimiter?: string;                  // ** or __
  emDelimiter?: string;                      // * or _
}
```

### Metadata Types
```typescript
interface JsMetadataConfig {
  extractHeaders?: boolean;              // h1-h6 elements
  extractLinks?: boolean;                // <a> elements
  extractImages?: boolean;               // <img> and inline SVG
  extractStructuredData?: boolean;       // JSON-LD, Microdata, RDFa
  maxStructuredDataSize?: number;        // Size limit (default: 1MB)
}

interface JsExtendedMetadata {
  document: JsDocumentMetadata;
  headers: JsHeaderMetadata[];
  links: JsLinkMetadata[];
  images: JsImageMetadata[];
  structuredData: JsStructuredData[];
}

interface JsDocumentMetadata {
  title?: string;
  description?: string;
  keywords: string[];
  author?: string;
  canonicalUrl?: string;
  language?: string;
  textDirection?: 'ltr' | 'rtl' | 'auto';
  openGraph: Record<string, string>;
  twitterCard: Record<string, string>;
  metaTags: Record<string, string>;
}
```

### Visitor Types
```typescript
interface Visitor {
  visitText?(ctx: NodeContext, text: string): VisitResult;
  visitLink?(ctx: NodeContext, href: string, text: string, title?: string): VisitResult;
  visitImage?(ctx: NodeContext, src: string, alt?: string, title?: string): VisitResult;
  visitHeading?(ctx: NodeContext, level: number, text: string, id?: string): VisitResult;
  visitCodeBlock?(ctx: NodeContext, lang?: string, code?: string): VisitResult;
  // ... 41 total methods for fine-grained control
}

interface NodeContext {
  nodeType: string;
  tagName: string;
  attributes: Record<string, string>;
  depth: number;
  indexInParent: number;
  parentTag: string | null;
  isInline: boolean;
}

type VisitResult =
  | { type: 'continue' }
  | { type: 'custom'; output: string }
  | { type: 'skip' }
  | { type: 'preserveHtml' }
  | { type: 'error'; message: string };
```

## Error Handling

```typescript
try {
  const markdown = convert(html);
} catch (error) {
  if (error instanceof Error) {
    console.error('Conversion failed:', error.message);
  }
}
```

Inputs with binary data (PDF bytes coerced to strings) raise errors with message: `Invalid input`.

## Examples

See comprehensive guides in the examples directory:

- **[Visitor Pattern](../../examples/visitor-pattern/)** - Custom callbacks, filtering, transformations, analytics
- **[Metadata Extraction](../../examples/metadata-extraction/)** - SEO metadata, TOC generation, link validation
- **[Performance](../../examples/performance/)** - Benchmarks, optimization strategies

## TypeScript Configuration

For strict type checking:

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "exactOptionalPropertyTypes": true,
    "noImplicitAny": true,
    "noImplicitThis": true,
    "strictNullChecks": true,
    "strictFunctionTypes": true,
    "strictPropertyInitialization": true,
    "noImplicitReturns": true
  }
}
```

All bindings are fully typed with no `any` types. Leverage TypeScript for compile-time safety.

## Performance

Benchmarks from Apple M4 (ops/sec):

| Document            | Size    | ops/sec |
| ------------------- | ------- | ------- |
| Small (Intro)       | 463 KB  | 627     |
| Medium (Python)     | 657 KB  | 460     |
| Large (Rust)        | 567 KB  | 554     |
| Lists (Timeline)    | 129 KB  | 3,137   |
| Tables (Countries)  | 360 KB  | 932     |

Run `task bench:harness -- --frameworks node` to benchmark locally.

## Links

- [GitHub](https://github.com/kreuzberg-dev/html-to-markdown)
- [npm Package](https://www.npmjs.com/package/@kreuzberg/html-to-markdown)
- [WASM Package](https://www.npmjs.com/package/@kreuzberg/html-to-markdown-wasm)
- [Discord Community](https://discord.gg/pXxagNK2zN)

## License

MIT
