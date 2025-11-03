# html-to-markdown-node

Native Node.js and Bun bindings for html-to-markdown using NAPI-RS v3.

Built on the shared Rust engine that powers the Python wheels, Ruby gem, WebAssembly package, and CLI – ensuring identical Markdown output across every language target.

High-performance HTML to Markdown conversion using native Rust code compiled to platform-specific binaries.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg)](https://crates.io/crates/html-to-markdown-rs)
[![npm version](https://badge.fury.io/js/html-to-markdown-node.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Gem Version](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)

## Performance

Native NAPI-RS bindings deliver **the fastest HTML to Markdown conversion** available in JavaScript.

### Benchmark Results (Apple M4)

| Document Type              | ops/sec    | Notes              |
| -------------------------- | ---------- | ------------------ |
| **Small (5 paragraphs)**   | **86,233** | Simple documents   |
| **Medium (25 paragraphs)** | **18,979** | Nested formatting  |
| **Large (100 paragraphs)** | **4,907**  | Complex structures |
| **Tables (20 tables)**     | **5,003**  | Table processing   |
| **Lists (500 items)**      | **1,819**  | Nested lists       |
| **Wikipedia (129KB)**      | **1,125**  | Real-world content |
| **Wikipedia (653KB)**      | **156**    | Large documents    |

**Average: ~18,162 ops/sec** across varied workloads.

### Comparison

- **vs WASM**: ~1.17× faster (native has zero startup time, direct memory access)
- **vs Python**: ~7.4× faster (avoids FFI overhead)
- **Best for**: Node.js and Bun server-side applications requiring maximum throughput

## Installation

### Node.js

```bash
npm install html-to-markdown-node
# or
yarn add html-to-markdown-node
# or
pnpm add html-to-markdown-node
```

### Bun

```bash
bun add html-to-markdown-node
```

## Usage

### Basic Conversion

```javascript
import { convert } from 'html-to-markdown-node';

const html = '<h1>Hello World</h1><p>This is <strong>fast</strong>!</p>';
const markdown = convert(html);
console.log(markdown);
// # Hello World
//
// This is **fast**!
```

### With Options

```typescript
import { convert } from 'html-to-markdown-node';

const markdown = convert(html, {
  headingStyle: 'Atx',
  codeBlockStyle: 'Backticks',
  listIndentWidth: 2,
  bullets: '-',
  wrap: true,
  wrapWidth: 80
});
```

### Preserve Complex HTML (NEW in v2.5)

```typescript
import { convert } from 'html-to-markdown-node';

const html = `
<h1>Report</h1>
<table>
  <tr><th>Name</th><th>Value</th></tr>
  <tr><td>Foo</td><td>Bar</td></tr>
</table>
`;

const markdown = convert(html, {
  preserveTags: ['table'] // Keep tables as HTML
});
// # Report
//
// <table>
//   <tr><th>Name</th><th>Value</th></tr>
//   <tr><td>Foo</td><td>Bar</td></tr>
// </table>
```

## TypeScript

Full TypeScript definitions included:

```typescript
import { convert, convertWithInlineImages, type JsConversionOptions } from 'html-to-markdown-node';

const options: JsConversionOptions = {
  headingStyle: 'Atx',
  codeBlockStyle: 'Backticks',
  listIndentWidth: 2,
  bullets: '-',
  wrap: true,
  wrapWidth: 80
};

const markdown = convert('<h1>Hello</h1>', options);
```

## Inline Images

Extract and decode inline images (data URIs, SVG):

```typescript
import { convertWithInlineImages } from 'html-to-markdown-node';

const html = '<img src="data:image/png;base64,iVBORw0..." alt="Logo">';

const result = convertWithInlineImages(html, null, {
  maxDecodedSizeBytes: 5 * 1024 * 1024, // 5MB
  inferDimensions: true,
  filenamePrefix: 'img_',
  captureSvg: true
});

console.log(result.markdown);
console.log(`Extracted ${result.inlineImages.length} images`);

for (const img of result.inlineImages) {
  console.log(`${img.filename}: ${img.format}, ${img.data.length} bytes`);
  // Save image data to disk
  require('fs').writeFileSync(img.filename, img.data);
}
```

## Supported Platforms

Pre-built native binaries are provided for:

| Platform    | Architectures                                       |
| ----------- | --------------------------------------------------- |
| **macOS**   | x64 (Intel), ARM64 (Apple Silicon)                  |
| **Linux**   | x64 (glibc/musl), ARM64 (glibc/musl), ARMv7 (glibc) |
| **Windows** | x64, ARM64                                          |

### Runtime Compatibility

✅ **Node.js** 18+ (LTS)
✅ **Bun** 1.0+ (full NAPI-RS support)
❌ **Deno** (use [html-to-markdown-wasm](https://www.npmjs.com/package/html-to-markdown-wasm) instead)

## When to Use

Choose `html-to-markdown-node` when:

- ✅ Running in Node.js or Bun
- ✅ Maximum performance is required
- ✅ Server-side conversion at scale

Use [`html-to-markdown-wasm`](https://www.npmjs.com/package/html-to-markdown-wasm) for:

- 🌐 Browser/client-side conversion
- 🦕 Deno runtime
- ☁️ Edge runtimes (Cloudflare Workers, Deno Deploy)
- 📦 Universal packages

Other runtimes:

- 🐍 Python: [`html-to-markdown`](https://pypi.org/project/html-to-markdown/)
- 💎 Ruby: [`html-to-markdown`](https://rubygems.org/gems/html-to-markdown)
- 🌐 WebAssembly: [`html-to-markdown-wasm`](https://www.npmjs.com/package/html-to-markdown-wasm)

## Configuration Options

See [ConversionOptions](https://github.com/Goldziher/html-to-markdown/tree/main/crates/html-to-markdown-node#types) for all available options including:

- Heading styles (ATX, underlined, ATX closed)
- Code block styles (indented, backticks, tildes)
- List formatting (indent width, bullet characters)
- Text escaping and formatting
- Tag preservation (`preserveTags`) and stripping (`stripTags`)
- Preprocessing for web scraping
- hOCR table extraction
- And more...

## Examples

### Preserving HTML Tags

Keep specific HTML tags in their original form instead of converting to Markdown:

```typescript
import { convert } from '@html-to-markdown/node';

const html = `
<p>Before table</p>
<table class="data">
    <tr><th>Name</th><th>Value</th></tr>
    <tr><td>Item 1</td><td>100</td></tr>
</table>
<p>After table</p>
`;

const markdown = convert(html, {
  preserveTags: ['table']
});

// Result includes the table as HTML:
// "Before table\n\n<table class=\"data\">...</table>\n\nAfter table\n"
```

Combine with `stripTags` for fine-grained control:

```typescript
const markdown = convert(html, {
  preserveTags: ['table', 'form'],  // Keep these as HTML
  stripTags: ['script', 'style']    // Remove these entirely
});
```

### Web Scraping

```javascript
const { convert } = require('html-to-markdown-node');

const scrapedHtml = await fetch('https://example.com').then(r => r.text());

const markdown = convert(scrapedHtml, {
  preprocessing: {
    enabled: true,
    preset: 'Aggressive',
    removeNavigation: true,
    removeForms: true
  },
  headingStyle: 'Atx',
  codeBlockStyle: 'Backticks'
});
```

### hOCR Document Processing

```javascript
const { convert } = require('html-to-markdown-node');
const fs = require('fs');

// OCR output from Tesseract in hOCR format
const hocrHtml = fs.readFileSync('scan.hocr', 'utf8');

// Automatically detects hOCR and reconstructs tables
const markdown = convert(hocrHtml, {
  hocrSpatialTables: true  // Enable spatial table reconstruction
});
```

## Links

- [GitHub Repository](https://github.com/Goldziher/html-to-markdown)
- [Full Documentation](https://github.com/Goldziher/html-to-markdown/blob/main/README.md)
- [WASM Package](https://www.npmjs.com/package/html-to-markdown-wasm)
- [Python Package](https://pypi.org/project/html-to-markdown/)
- [Rust Crate](https://crates.io/crates/html-to-markdown-rs)

## License

MIT
