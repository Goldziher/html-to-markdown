# html-to-markdown-wasm

Universal HTML to Markdown converter using WebAssembly.

Powered by the same Rust engine as the Node.js, Python, and Ruby bindings, so Markdown output stays identical regardless of runtime.

Runs anywhere: Node.js, Deno, Bun, browsers, and edge runtimes.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg)](https://crates.io/crates/html-to-markdown-rs)
[![npm version](https://badge.fury.io/js/html-to-markdown-wasm.svg)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Gem Version](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)

## Performance

Universal WebAssembly bindings with **excellent performance** across all JavaScript runtimes.

### Benchmark Results (Apple M4)

| Document Type              | ops/sec    | Notes              |
| -------------------------- | ---------- | ------------------ |
| **Small (5 paragraphs)**   | **70,300** | Simple documents   |
| **Medium (25 paragraphs)** | **15,282** | Nested formatting  |
| **Large (100 paragraphs)** | **3,836**  | Complex structures |
| **Tables (20 tables)**     | **3,748**  | Table processing   |
| **Lists (500 items)**      | **1,391**  | Nested lists       |
| **Wikipedia (129KB)**      | **1,022**  | Real-world content |
| **Wikipedia (653KB)**      | **147**    | Large documents    |

**Average: ~15,536 ops/sec** across varied workloads.

### Comparison

- **vs Native NAPI**: ~1.17× slower (WASM has minimal overhead)
- **vs Python**: ~6.3× faster (no FFI overhead)
- **Best for**: Universal deployment (browsers, Deno, edge runtimes, cross-platform apps)

## Installation

### npm / Yarn / pnpm

```bash
npm install html-to-markdown-wasm
# or
yarn add html-to-markdown-wasm
# or
pnpm add html-to-markdown-wasm
```

### Deno

```typescript
// Via npm specifier
import { convert } from "npm:html-to-markdown-wasm";
```

## Usage

### Basic Conversion

```javascript
import { convert } from 'html-to-markdown-wasm';

const html = '<h1>Hello World</h1><p>This is <strong>fast</strong>!</p>';
const markdown = convert(html);
console.log(markdown);
// # Hello World
//
// This is **fast**!
```

### With Options

```typescript
import { convert } from 'html-to-markdown-wasm';

const markdown = convert(html, {
  headingStyle: 'atx',
  codeBlockStyle: 'backticks',
  listIndentWidth: 2,
  bullets: '-',
  wrap: true,
  wrapWidth: 80
});
```

### Preserve Complex HTML (NEW in v2.5)

```typescript
import { convert } from 'html-to-markdown-wasm';

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
```

### Deno

```typescript
import { convert } from "npm:html-to-markdown-wasm";

const html = await Deno.readTextFile("input.html");
const markdown = convert(html, { headingStyle: "atx" });
await Deno.writeTextFile("output.md", markdown);
```

> **Performance Tip:** For Node.js/Bun, use [html-to-markdown-node](https://www.npmjs.com/package/html-to-markdown-node) for 1.17× better performance with native bindings.

### Browser (ESM)

```html
<!DOCTYPE html>
<html>
<head>
  <title>HTML to Markdown</title>
</head>
<body>
  <script type="module">
    import init, { convert } from 'https://unpkg.com/html-to-markdown-wasm/dist-web/html_to_markdown_wasm.js';

    // Initialize WASM module
    await init();

    const html = '<h1>Hello World</h1><p>This runs in the <strong>browser</strong>!</p>';
    const markdown = convert(html, { headingStyle: 'atx' });

    console.log(markdown);
    document.body.innerHTML = `<pre>${markdown}</pre>`;
  </script>
</body>
</html>
```

### Vite / Webpack / Bundlers

```typescript
import { convert } from 'html-to-markdown-wasm';

const markdown = convert('<h1>Hello</h1>', {
  headingStyle: 'atx',
  codeBlockStyle: 'backticks'
});
```

### Cloudflare Workers

```typescript
import { convert } from 'html-to-markdown-wasm';

export default {
  async fetch(request: Request): Promise<Response> {
    const html = await request.text();
    const markdown = convert(html, { headingStyle: 'atx' });

    return new Response(markdown, {
      headers: { 'Content-Type': 'text/markdown' }
    });
  }
};
```

## TypeScript

Full TypeScript support with type definitions:

```typescript
import {
  convert,
  convertWithInlineImages,
  WasmInlineImageConfig,
  type WasmConversionOptions
} from 'html-to-markdown-wasm';

const options: WasmConversionOptions = {
  headingStyle: 'atx',
  codeBlockStyle: 'backticks',
  listIndentWidth: 2,
  wrap: true,
  wrapWidth: 80
};

const markdown = convert('<h1>Hello</h1>', options);
```

## Inline Images

Extract and decode inline images (data URIs, SVG):

```typescript
import { convertWithInlineImages, WasmInlineImageConfig } from 'html-to-markdown-wasm';

const html = '<img src="data:image/png;base64,iVBORw0..." alt="Logo">';

const config = new WasmInlineImageConfig(5 * 1024 * 1024); // 5MB max
config.inferDimensions = true;
config.filenamePrefix = 'img_';
config.captureSvg = true;

const result = convertWithInlineImages(html, null, config);

console.log(result.markdown);
console.log(`Extracted ${result.inlineImages.length} images`);

for (const img of result.inlineImages) {
  console.log(`${img.filename}: ${img.format}, ${img.data.length} bytes`);
  // img.data is a Uint8Array - save to file or upload
}
```

## Build Targets

Three build targets are provided for different environments:

| Target      | Path                              | Use Case                       |
| ----------- | --------------------------------- | ------------------------------ |
| **Bundler** | `html-to-markdown-wasm`           | Webpack, Vite, Rollup, esbuild |
| **Node.js** | `html-to-markdown-wasm/dist-node` | Node.js, Bun (CommonJS/ESM)    |
| **Web**     | `html-to-markdown-wasm/dist-web`  | Direct browser ESM imports     |

## Runtime Compatibility

| Runtime                   | Support                      | Package        |
| ------------------------- | ---------------------------- | -------------- |
| ✅ **Node.js** 18+        | Full support                 | `dist-node`    |
| ✅ **Deno**               | Full support                 | npm: specifier |
| ✅ **Bun**                | Full support (prefer native) | Default export |
| ✅ **Browsers**           | Full support                 | `dist-web`     |
| ✅ **Cloudflare Workers** | Full support                 | Default export |
| ✅ **Deno Deploy**        | Full support                 | npm: specifier |

## When to Use

Choose `html-to-markdown-wasm` when:

- 🌐 Running in browsers or edge runtimes
- 🦕 Using Deno
- ☁️ Deploying to Cloudflare Workers, Deno Deploy
- 📦 Building universal libraries
- 🔄 Need consistent behavior across all platforms

Use [html-to-markdown-node](https://www.npmjs.com/package/html-to-markdown-node) for:

- ⚡ Maximum performance in Node.js/Bun (~3× faster)
- 🖥️ Server-side only applications

## Configuration Options

See the [TypeScript definitions](./dist-node/html_to_markdown_wasm.d.ts) for all available options:

- Heading styles (atx, underlined, atxClosed)
- Code block styles (indented, backticks, tildes)
- List formatting (indent width, bullet characters)
- Text escaping and formatting
- Tag preservation (`preserveTags`) and stripping (`stripTags`)
- Preprocessing for web scraping
- hOCR table extraction
- And more...

## Examples

### Preserving HTML Tags

Keep specific HTML tags in their original form:

```typescript
import { convert } from '@html-to-markdown/wasm';

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

// Result includes the table as HTML
```

Combine with `stripTags`:

```typescript
const markdown = convert(html, {
  preserveTags: ['table', 'form'],  // Keep as HTML
  stripTags: ['script', 'style']    // Remove entirely
});
```

### Deno Web Server

```typescript
import { convert } from "npm:html-to-markdown-wasm";

Deno.serve((req) => {
  const url = new URL(req.url);

  if (url.pathname === "/convert" && req.method === "POST") {
    const html = await req.text();
    const markdown = convert(html, { headingStyle: "atx" });

    return new Response(markdown, {
      headers: { "Content-Type": "text/markdown" }
    });
  }

  return new Response("Not found", { status: 404 });
});
```

### Browser File Conversion

```html
<input type="file" id="htmlFile" accept=".html">
<button onclick="convertFile()">Convert to Markdown</button>
<pre id="output"></pre>

<script type="module">
  import init, { convert } from 'https://unpkg.com/html-to-markdown-wasm/dist-web/html_to_markdown_wasm.js';

  await init();

  window.convertFile = async () => {
    const file = document.getElementById('htmlFile').files[0];
    const html = await file.text();
    const markdown = convert(html, { headingStyle: 'atx' });
    document.getElementById('output').textContent = markdown;
  };
</script>
```

### Web Scraping (Deno)

```typescript
import { convert } from "npm:html-to-markdown-wasm";

const response = await fetch("https://example.com");
const html = await response.text();

const markdown = convert(html, {
  preprocessing: {
    enabled: true,
    preset: "aggressive",
    removeNavigation: true,
    removeForms: true
  },
  headingStyle: "atx",
  codeBlockStyle: "backticks"
});

console.log(markdown);
```

## Other Runtimes

The same Rust engine ships as native bindings for other ecosystems:

- 🖥️ Node.js / Bun: [`html-to-markdown-node`](https://www.npmjs.com/package/html-to-markdown-node)
- 🐍 Python: [`html-to-markdown`](https://pypi.org/project/html-to-markdown/)
- 💎 Ruby: [`html-to-markdown`](https://rubygems.org/gems/html-to-markdown)
- 🦀 Rust crate & CLI: [`html-to-markdown-rs`](https://crates.io/crates/html-to-markdown-rs)

## Links

- [GitHub Repository](https://github.com/Goldziher/html-to-markdown)
- [Full Documentation](https://github.com/Goldziher/html-to-markdown/blob/main/README.md)
- [Native Node Package](https://www.npmjs.com/package/html-to-markdown-node)
- [Python Package](https://pypi.org/project/html-to-markdown/)
- [Rust Crate](https://crates.io/crates/html-to-markdown-rs)

## License

MIT
