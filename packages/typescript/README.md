# html-to-markdown (TypeScript)

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown.svg)](https://crates.io/crates/html-to-markdown)
[![npm (node)](https://badge.fury.io/js/html-to-markdown-node.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://badge.fury.io/js/html-to-markdown-wasm.svg)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![NuGet](https://img.shields.io/nuget/v/HtmlToMarkdown.svg)](https://www.nuget.org/packages/HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)

High-performance HTML to Markdown converter for Node.js and Bun. This package wraps the
native `html-to-markdown-node` bindings and adds a TypeScript-friendly API plus a
cross-platform CLI.

```bash
npm install html-to-markdown
```

## Usage

```ts
import { convert } from 'html-to-markdown';

const markdown = convert('<h1>Hello</h1>');
```

The package re-exports all conversion options exposed by the native bindings. See the
[core documentation](https://github.com/Goldziher/html-to-markdown) for complete
option descriptions.

### File helpers

```ts
import { convertFile } from 'html-to-markdown';

const markdown = await convertFile('input.html');
```

### CLI

```bash
npx html-to-markdown --input input.html --output output.md
```

Use `npx html-to-markdown --help` for full usage information.

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
