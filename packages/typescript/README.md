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
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown)
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

## Usage

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

### Metadata extraction

```ts
import { convertWithMetadata } from 'html-to-markdown-node';

const html = `
  <html>
    <head><title>Example</title></head>
    <body>
      <h1 id="welcome">Welcome</h1>
      <a href="https://example.com" rel="nofollow">Example</a>
      <img src="https://example.com/img.jpg" alt="Hero" width="640" height="480">
    </body>
  </html>
`;

const { markdown, metadata } = await convertWithMetadata(
  html,
  { headingStyle: 'Atx' },
  { extract_links: true, extract_images: true, extract_headers: true },
);

console.log(markdown);
console.log(metadata.document.title); // "Example"
console.log(metadata.links[0].rel);  // ["nofollow"]
console.log(metadata.images[0].dimensions); // [640, 480]
```

### File helpers

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
