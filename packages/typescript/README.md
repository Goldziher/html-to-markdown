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
    <img src="https://img.shields.io/badge/Go-v2.19.0-007ec6" alt="Go">
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


High-performance HTML to Markdown converter for Node.js and Bun with full TypeScript support.
This package wraps native `@kreuzberg/html-to-markdown-node` bindings and provides a type-safe API.


## Installation

```bash
npm install @kreuzberg/html-to-markdown
```



Requires Node.js 18+ or Bun. Native bindings provide superior performance.

**npm:**
```bash
npm install @kreuzberg/html-to-markdown
```

**pnpm:**
```bash
pnpm add @kreuzberg/html-to-markdown
```

**yarn:**
```bash
yarn add @kreuzberg/html-to-markdown
```

**bun:**
```bash
bun add @kreuzberg/html-to-markdown
```

Alternatively, use the WebAssembly version for browser/edge environments:

```bash
npm install @kreuzberg/html-to-markdown-wasm
```




# Migration Guide: TypeScript v2.18.x → v2.19.0

## Breaking Change: Scoped npm Packages

In v2.19.0, npm packages were moved to the `@kreuzberg` scope to align with the Kreuzberg.dev organization.

### Package Installation Update

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

### Import Statement Update

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

### TypeScript Declaration Update

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

### Deno Update

**Before:**
```typescript
import { convert } from "npm:html-to-markdown-wasm";
```

**After:**
```typescript
import { convert } from "npm:@kreuzberg/html-to-markdown-wasm";
```

## Summary of Changes

- All npm packages now use `@kreuzberg` scope
- `html-to-markdown-node` → `@kreuzberg/html-to-markdown-node`
- `html-to-markdown-wasm` → `@kreuzberg/html-to-markdown-wasm`
- TypeScript types and APIs are identical
- No functional changes to the library




## Performance Snapshot

Apple M4 • Real Wikipedia documents • `convert()` (TypeScript (Node.js))

| Document | Size | Latency | Throughput |
| -------- | ---- | ------- | ---------- |
| Lists (Timeline) | 129KB | 0.58ms | 222 MB/s |
| Tables (Countries) | 360KB | 1.89ms | 190 MB/s |
| Mixed (Python wiki) | 656KB | 4.21ms | 156 MB/s |


See [Performance Guide](../../examples/performance/) for detailed benchmarks.


## Quick Start

Basic conversion:

```typescript
import { convert } from '@kreuzberg/html-to-markdown';

const markdown: string = convert('<h1>Hello World</h1>');
console.log(markdown); // # Hello World
```



With conversion options:

```typescript
import { convert, ConversionOptions } from '@kreuzberg/html-to-markdown';

const options: ConversionOptions = {
  headingStyle: 'atx',
  listIndentWidth: 2,
  wrap: true,
};

const markdown = convert('<h1>Title</h1><p>Content</p>', options);
```






## API Reference

### Core Functions


**`convert(html: string, options?: ConversionOptions): string`**

Basic HTML-to-Markdown conversion. Fast and simple.

**`convertWithMetadata(html: string, options?: ConversionOptions, config?: MetadataConfig): { markdown: string; metadata: Metadata }`**

Extract Markdown plus metadata (headers, links, images, structured data) in a single pass. See [Metadata Extraction Guide](../../examples/metadata-extraction/).

**`convertWithVisitor(html: string, options: { visitor: Visitor } & ConversionOptions): string`**

Customize conversion with visitor callbacks for element interception. See [Visitor Pattern Guide](../../examples/visitor-pattern/).

**`convertWithAsyncVisitor(html: string, options: { visitor: AsyncVisitor } & ConversionOptions): Promise<string>`**

Async version of visitor pattern for I/O operations.

**`convertWithInlineImages(html: string, config?: InlineImageConfig): { markdown: string; images: ImageData[]; warnings: string[] }`**

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



## Metadata Extraction

The metadata extraction feature enables comprehensive document analysis during conversion. Extract document properties, headers, links, images, and structured data in a single pass.

**Use Cases:**
- **SEO analysis** – Extract title, description, Open Graph tags, Twitter cards
- **Table of contents generation** – Build structured outlines from heading hierarchy
- **Content migration** – Document all external links and resources
- **Accessibility audits** – Check for images without alt text, empty links, invalid heading hierarchy
- **Link validation** – Classify and validate anchor, internal, external, email, and phone links

**Zero Overhead When Disabled:** Metadata extraction adds negligible overhead and happens during the HTML parsing pass. Disable unused metadata types in `MetadataConfig` to optimize further.

### Example: Quick Start


```typescript
import { convertWithMetadata } from 'html-to-markdown';

const html = '<h1>Article</h1><img src="test.jpg" alt="test">';
const { markdown, metadata } = convertWithMetadata(html);

console.log(metadata.document.title);      // Document title
console.log(metadata.headers);             // All h1-h6 elements
console.log(metadata.links);               // All hyperlinks
console.log(metadata.images);              // All images with alt text
console.log(metadata.structuredData);      // JSON-LD, Microdata, RDFa
```



For detailed examples including SEO extraction, table-of-contents generation, link validation, and accessibility audits, see the [Metadata Extraction Guide](../../examples/metadata-extraction/).




## Visitor Pattern

The visitor pattern enables custom HTML→Markdown conversion logic by providing callbacks for specific HTML elements during traversal. Use visitors to transform content, filter elements, validate structure, or collect analytics.

**Use Cases:**
- **Custom Markdown dialects** – Convert to Obsidian, Notion, or other flavors
- **Content filtering** – Remove tracking pixels, ads, or unwanted elements
- **URL rewriting** – Rewrite CDN URLs, add query parameters, validate links
- **Accessibility validation** – Check alt text, heading hierarchy, link text
- **Analytics** – Track element usage, link destinations, image sources

**Supported Visitor Methods:** 40+ callbacks for text, inline elements, links, images, headings, lists, blocks, and tables.

### Example: Quick Start


```typescript
import { convertWithVisitor, type Visitor, type NodeContext, type VisitResult } from 'html-to-markdown';

const visitor: Visitor = {
  visitLink(ctx: NodeContext, href: string, text: string, title?: string): VisitResult {
    // Rewrite CDN URLs
    if (href.startsWith('https://old-cdn.com')) {
      href = href.replace('https://old-cdn.com', 'https://new-cdn.com');
    }
    return { type: 'custom', output: `[${text}](${href})` };
  },

  visitImage(ctx: NodeContext, src: string, alt?: string, title?: string): VisitResult {
    // Skip tracking pixels
    if (src.includes('tracking')) {
      return { type: 'skip' };
    }
    return { type: 'continue' };
  },
};

const html = '<a href="https://old-cdn.com/file.pdf">Download</a>';
const markdown = convertWithVisitor(html, { visitor });
```

Async support:
```typescript
import { convertWithAsyncVisitor, type AsyncVisitor } from 'html-to-markdown';

const asyncVisitor: AsyncVisitor = {
  async visitLink(ctx, href, text, title) {
    const isValid = await validateUrl(href);
    return isValid ? { type: 'continue' } : { type: 'error', message: `Broken link: ${href}` };
  },
};

const markdown = await convertWithAsyncVisitor(html, { visitor: asyncVisitor });
```



For comprehensive examples including content filtering, link footnotes, accessibility validation, and asynchronous URL validation, see the [Visitor Pattern Guide](../../examples/visitor-pattern/).



## Examples

- [Visitor Pattern Guide](../../examples/visitor-pattern/)
- [Metadata Extraction Guide](../../examples/metadata-extraction/)
- [Performance Guide](../../examples/performance/)

## Links

- **GitHub:** [github.com/kreuzberg-dev/html-to-markdown](https://github.com/kreuzberg-dev/html-to-markdown)

- **npm:** [npmjs.com/@kreuzberg/html-to-markdown-node](https://www.npmjs.com/package/@kreuzberg/html-to-markdown-node)
- **WASM:** [npmjs.com/@kreuzberg/html-to-markdown-wasm](https://www.npmjs.com/package/@kreuzberg/html-to-markdown-wasm)

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
