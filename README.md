# html-to-markdown

High-performance HTML → Markdown conversion powered by Rust. Shipping as a Rust crate, Python package, Node.js bindings, WebAssembly, and standalone CLI with identical rendering behaviour.

[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![npm version](https://badge.fury.io/js/html-to-markdown.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg)](https://crates.io/crates/html-to-markdown-rs)
[![Python Versions](https://img.shields.io/pypi/pyversions/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

## Documentation

- **JavaScript/TypeScript guides**:
    - Node.js/Bun (native) – [Node.js README](https://github.com/Goldziher/html-to-markdown/tree/main/crates/html-to-markdown-node)
    - WebAssembly (universal) – [WASM README](https://github.com/Goldziher/html-to-markdown/tree/main/crates/html-to-markdown-wasm)
- **Python guide** – [Python README](https://github.com/Goldziher/html-to-markdown/blob/main/README_PYPI.md)
- **Rust guide** – [Rust README](https://github.com/Goldziher/html-to-markdown/tree/main/crates/html-to-markdown)
- **Contributing** – [CONTRIBUTING.md](https://github.com/Goldziher/html-to-markdown/blob/main/CONTRIBUTING.md) ⭐ Start here!
- **Changelog** – [CHANGELOG.md](https://github.com/Goldziher/html-to-markdown/blob/main/CHANGELOG.md)

## Installation

| Target                      | Command                                                                   |
| --------------------------- | ------------------------------------------------------------------------- |
| **Node.js/Bun** (native)    | `npm install html-to-markdown-node`                                       |
| **WebAssembly** (universal) | `npm install html-to-markdown-wasm`                                       |
| **Deno**                    | `import { convert } from "npm:html-to-markdown-wasm"`                     |
| **Python** (bindings + CLI) | `pip install html-to-markdown`                                            |
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

**Performance:** Native bindings average ~19k ops/sec, WASM averages ~16k ops/sec (benchmarked on complex real-world documents).

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

| Document Type              | Node.js (NAPI) | WASM   | Python (PyO3) | Speedup (Node vs Python) |
| -------------------------- | -------------- | ------ | ------------- | ------------------------ |
| **Small (5 paragraphs)**   | 86,233         | 70,300 | 8,443         | **10.2×**                |
| **Medium (25 paragraphs)** | 18,979         | 15,282 | 1,846         | **10.3×**                |
| **Large (100 paragraphs)** | 4,907          | 3,836  | 438           | **11.2×**                |
| **Tables (complex)**       | 5,003          | 3,748  | 4,829         | 1.0×                     |
| **Lists (nested)**         | 1,819          | 1,391  | 1,165         | **1.6×**                 |
| **Wikipedia (129KB)**      | 1,125          | 1,022  | -             | -                        |
| **Wikipedia (653KB)**      | 156            | 147    | -             | -                        |

### Average Performance Summary

| Implementation        | Avg ops/sec      | vs WASM      | vs Python       | Best For                          |
| --------------------- | ---------------- | ------------ | --------------- | --------------------------------- |
| **Node.js (NAPI-RS)** | **18,162**       | 1.17× faster | **7.4× faster** | Maximum throughput in Node.js/Bun |
| **WebAssembly**       | **15,536**       | baseline     | **6.3× faster** | Universal (Deno, browsers, edge)  |
| **Python (PyO3)**     | **2,465**        | 6.3× slower  | baseline        | Python ecosystem integration      |
| **Rust CLI/Binary**   | **150-210 MB/s** | -            | -               | Standalone processing             |

### Key Insights

- **JavaScript bindings are fastest**: Native Node.js bindings achieve ~18k ops/sec average, with WASM close behind at ~16k ops/sec
- **Python is 6-10× slower**: Despite using the same Rust core, PyO3 FFI overhead significantly impacts Python performance
- **Small documents**: Both JS implementations reach 70-90k ops/sec on simple HTML
- **Large documents**: Performance gap widens with complexity

**Note on Python performance**: The current Python bindings have optimization opportunities. The v2 API with direct `convert()` calls performs best; avoid the v1 compatibility layer for performance-critical applications.

## Compatibility (v1 → v2)

- V2’s Rust core sustains **150–210 MB/s** throughput; V1 averaged **≈ 2.5 MB/s** in its Python/BeautifulSoup implementation (60–80× faster).
- The Python package offers a compatibility shim in `html_to_markdown.v1_compat` (`convert_to_markdown`, `convert_to_markdown_stream`, `markdownify`). Details and keyword mappings live in [Python README](https://github.com/Goldziher/html-to-markdown/blob/main/README_PYPI.md#v1-compatibility).
- CLI flag changes, option renames, and other breaking updates are summarised in [CHANGELOG](https://github.com/Goldziher/html-to-markdown/blob/main/CHANGELOG.md#breaking-changes).

## Community

- Chat with us on [Discord](https://discord.gg/pXxagNK2zN)
- Explore the broader [Kreuzberg](https://kreuzberg.dev) document-processing ecosystem
- Sponsor development via [GitHub Sponsors](https://github.com/sponsors/Goldziher)
