# html-to-markdown

<img width="1128" height="191" alt="Linkedin- Banner (1)" src="https://github.com/user-attachments/assets/f8e91036-20a5-40f9-9fcc-9e6c6e15f1f5" />



High-performance HTML → Markdown conversion powered by Rust. Shipping as a Rust crate, Python package, PHP extension, Ruby gem, Elixir Rustler NIF, Node.js bindings, WebAssembly, and standalone CLI with identical rendering behaviour. 

Part of the Kreuzberg.dev document intelligence ecosystem. Kreuzberg is a polyglot document intelligence framework with a fast Rust core. We build tools that help developers extract, process, and understand documents at scale, from PDFs to Office files, images, archives, emails, in 50+ formats. We've set out to make high-performance document intelligence faster and more ecological.

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

## Quick Start

**Node.js / Bun (Native - Fastest):**

```typescript
import { convert } from 'html-to-markdown-node';

const html = '<h1>Hello</h1><p>Rust ❤️ Markdown</p>';
const markdown = convert(html, {
  headingStyle: 'Atx',
  codeBlockStyle: 'Backticks',
  wrap: true,
  preserveTags: ['table'],
});
```

**Python:**

```python
from html_to_markdown import convert

html = '<h1>Hello</h1><p>Rust ❤️ Markdown</p>'
markdown = convert(html, heading_style='Atx', wrap=True)
```

**Ruby:**

```ruby
require 'html_to_markdown'

html = '<h1>Hello</h1><p>Rust ❤️ Markdown</p>'
markdown = HtmlToMarkdown.convert(html, heading_style: :atx, wrap: true)
```

Full language guides: See [Language Guides](#language-guides) below.

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

## Performance

Benchmarked on Apple M4 using the shared fixture harness in `tools/benchmark-harness`.

### Comparative Throughput (Median Across Fixtures)

| Runtime | Median ops/sec | Median throughput (MB/s) | Peak memory (MB) | Successes |
| ------- | -------------- | ------------------------ | ---------------- | --------- |
| Rust | 1,060.3 | 116.4 | 171.3 | 56/56 |
| Go | 1,496.3 | 131.1 | 22.9 | 16/16 |
| Ruby | 2,155.5 | 300.4 | 280.3 | 48/48 |
| PHP | 2,357.7 | 308.0 | 223.5 | 48/48 |
| Elixir | 1,564.1 | 269.1 | 384.7 | 48/48 |
| C# | 1,234.2 | 272.4 | 187.8 | 16/16 |
| Java | 1,298.7 | 167.1 | 527.2 | 16/16 |
| WASM | 1,485.8 | 157.6 | 95.3 | 48/48 |
| Node.js (NAPI) | 2,054.2 | 306.5 | 95.4 | 48/48 |
| Python (PyO3) | 3,120.3 | 307.5 | 83.5 | 48/48 |

Use `task bench:harness` to regenerate throughput numbers. See [Performance Guide](./examples/performance/) for benchmarking strategies and optimization tips.

## Language Guides

Complete documentation with examples for each language:

- **Python** – [README](./packages/python/README.md) | PyO3 bindings, metadata extraction, inline images
- **JavaScript/TypeScript** – [Node.js](./crates/html-to-markdown-node/README.md) | [TypeScript](./packages/typescript/README.md) | [WASM](./crates/html-to-markdown-wasm/README.md)
- **Ruby** – [README](./packages/ruby/README.md) | Magnus bindings, RBS type definitions, Steep checking
- **PHP** – [Package](./packages/php/README.md) | [Extension (PIE)](./packages/php-ext/README.md) | ext-php-rs extension
- **Go** – [README](./packages/go/README.md) | FFI bindings with cgo
- **Java** – [README](./packages/java/README.md) | Panama FFI, Maven/Gradle setup
- **C#/.NET** – [README](./packages/csharp/README.md) | P/Invoke FFI, NuGet distribution
- **Elixir** – [README](./packages/elixir/README.md) | Rustler NIF bindings
- **Rust** – [README](./crates/html-to-markdown/README.md) | Core library, error handling, advanced features

## Feature Guides

### Visitor Pattern
Customize HTML→Markdown conversion with callbacks for specific elements. Use cases: domain-specific dialects, content filtering, URL rewriting, accessibility validation.

**→ [Full Guide with Examples](./examples/visitor-pattern/)** (Python, TypeScript, Ruby)

### Metadata Extraction
Extract comprehensive metadata during conversion: title, description, headers, links, images, structured data. Use cases: SEO extraction, TOC generation, link validation, accessibility auditing, content migration.

**→ [Full Guide with Examples](./examples/metadata-extraction/)** (Python, TypeScript, Ruby)

### Performance & Benchmarking
Understand performance characteristics, run benchmarks, optimize for your use case. Includes benchmarking tools, memory profiling, streaming strategies, and optimization tips.

**→ [Full Guide](./examples/performance/)**

## Examples

Explore working code examples in multiple languages:

| Example | Path | Languages |
| ------- | ---- | --------- |
| **Visitor Pattern** | [examples/visitor-pattern/](./examples/visitor-pattern/) | Python, TypeScript, Ruby |
| **Metadata Extraction** | [examples/metadata-extraction/](./examples/metadata-extraction/) | Python, TypeScript, Ruby |
| **Performance** | [examples/performance/](./examples/performance/) | Benchmarks, profiling, optimization |

## Testing

Run the test suite locally:

```bash
# All core test suites (Rust, Python, Ruby, Node, PHP, Go, C#, Elixir, Java)
task test

# Run the Wasmtime-backed WASM integration tests
task wasm:test:wasmtime
```

## Compatibility (v1 → v2)

- V2's Rust core sustains **150–210 MB/s** throughput; V1 averaged **≈ 2.5 MB/s** (60–80× faster).
- Python compatibility shim available in `html_to_markdown.v1_compat` (deprecated; emits warnings; plan migrations now). See [Python README](./packages/python/README.md#v1-compatibility) for keyword mappings.
- CLI flag changes and other breaking updates in [CHANGELOG](./CHANGELOG.md#breaking-changes).

## Community

- **Discord** – [Join our community](https://discord.gg/pXxagNK2zN)
- **Ecosystem** – Explore [Kreuzberg](https://kreuzberg.dev) document-processing tools
- **Contribute** – [CONTRIBUTING.md](./CONTRIBUTING.md)
- **Sponsor** – [GitHub Sponsors](https://github.com/sponsors/Goldziher)
- **Changelog** – [Version history](./CHANGELOG.md)

## License

MIT License – see [LICENSE](./LICENSE) for details.
