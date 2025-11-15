# html-to-markdown

High-performance HTML → Markdown conversion powered by Rust. Shipping as a Rust crate, Python package, PHP extension, Ruby gem, Elixir Rustler NIF, Node.js bindings, WebAssembly, and standalone CLI with identical rendering behaviour.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown.svg)](https://crates.io/crates/html-to-markdown)
[![npm (node)](https://badge.fury.io/js/html-to-markdown-node.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://badge.fury.io/js/html-to-markdown-wasm.svg)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/HtmlToMarkdown.svg)](https://www.nuget.org/packages/HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown)
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
- **Highly Configurable**: Control heading styles, code block fences, list formatting, whitespace handling, and HTML sanitization
- **Tag Preservation**: Keep specific HTML tags unconverted when markdown isn't expressive enough
- **Secure by Default**: Built-in HTML sanitization prevents malicious content
- **Consistent Output**: Identical markdown rendering across all language bindings

## Documentation

- **JavaScript/TypeScript guides**:
    - Node.js/Bun (native) – [Node.js README](https://github.com/Goldziher/html-to-markdown/blob/main/crates/html-to-markdown-node/README.md)
    - WebAssembly (universal) – [WASM README](https://github.com/Goldziher/html-to-markdown/blob/main/crates/html-to-markdown-wasm/README.md)
    - TypeScript wrapper – [TypeScript README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/typescript/README.md)
- **Python guide** – [Python README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/python/README.md)
- **PHP guides**:
    - PHP wrapper package – [PHP README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/php/README.md)
    - PHP extension (PIE) – [Extension README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/php-ext/README.md)
- **Ruby guide** – [Ruby README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/ruby/README.md)
- **Elixir guide** – [Elixir README](https://github.com/Goldziher/html-to-markdown/blob/main/packages/elixir/README.md)
- **Rust guide** – [Rust README](https://github.com/Goldziher/html-to-markdown/blob/main/crates/html-to-markdown/README.md)
- **Contributing** – [CONTRIBUTING.md](https://github.com/Goldziher/html-to-markdown/blob/main/CONTRIBUTING.md) ⭐ Start here!
- **Changelog** – [CHANGELOG.md](https://github.com/Goldziher/html-to-markdown/blob/main/CHANGELOG.md)

## Installation

| Target                      | Command                                                                   |
| --------------------------- | ------------------------------------------------------------------------- |
| **Node.js/Bun** (native)    | `npm install html-to-markdown-node`                                       |
| **WebAssembly** (universal) | `npm install html-to-markdown-wasm`                                       |
| **Deno**                    | `import { convert } from "npm:html-to-markdown-wasm"`                     |
| **Python** (bindings + CLI) | `pip install html-to-markdown`                                            |
| **PHP** (extension + helpers) | `pie install goldziher/html-to-markdown`<br>`composer require html-to-markdown/extension` |
| **Ruby** gem                | `bundle add html-to-markdown` or `gem install html-to-markdown`           |
| **Elixir** (Rustler NIF)    | `{:html_to_markdown, "~> 2.8"}`                                           |
| **Rust** crate              | `cargo add html-to-markdown-rs`                                           |
| Rust CLI                    | `cargo install html-to-markdown-cli`                                      |
| Homebrew CLI                | `brew tap goldziher/tap`<br>`brew install html-to-markdown`               |
| Releases                    | [GitHub Releases](https://github.com/Goldziher/html-to-markdown/releases) |

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

### CLI

```bash
# Convert a file
html-to-markdown input.html > output.md

# Stream from stdin
curl https://example.com | html-to-markdown > output.md

# Apply options
html-to-markdown --heading-style atx --list-indent-width 2 input.html
```

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
