# html-to-markdown

High-performance HTML → Markdown conversion powered by Rust. Shipping as a Rust crate, Python package, Node.js bindings, WebAssembly, and standalone CLI with identical rendering behaviour.

[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![npm version](https://badge.fury.io/js/html-to-markdown.svg)](https://www.npmjs.com/package/html-to-markdown)
[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg)](https://crates.io/crates/html-to-markdown-rs)
[![Python Versions](https://img.shields.io/pypi/pyversions/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

## Documentation

- **JavaScript/TypeScript guide** – [packages/html-to-markdown/README.md](packages/html-to-markdown/README.md)
- **Python guide** – [README_PYPI.md](README_PYPI.md)
- **Rust guide** – [crates/html-to-markdown/README.md](crates/html-to-markdown/README.md)
- **Repository structure** – [STRUCTURE.md](STRUCTURE.md)
- **Contributing** – [CONTRIBUTING.md](CONTRIBUTING.md) ⭐ Start here!
- **Changelog** – [CHANGELOG.md](CHANGELOG.md)

## Installation

| Target                      | Command                                                                   |
| --------------------------- | ------------------------------------------------------------------------- |
| **JavaScript/TypeScript**   | `npm install html-to-markdown`                                            |
| Node.js native (NAPI-RS)    | `npm install @html-to-markdown/node`                                      |
| WebAssembly (browsers)      | `npm install @html-to-markdown/wasm`                                      |
| **Python** (bindings + CLI) | `pip install html-to-markdown`                                            |
| **Rust** crate              | `cargo add html-to-markdown-rs`                                           |
| Rust CLI                    | `cargo install html-to-markdown-cli`                                      |
| Homebrew CLI                | `brew tap goldziher/tap`<br>`brew install html-to-markdown`               |
| Releases                    | [GitHub Releases](https://github.com/Goldziher/html-to-markdown/releases) |

## Quick Start

### JavaScript/TypeScript

```typescript
import { convert } from 'html-to-markdown';

const html = '<h1>Hello</h1><p>Rust ❤️ Markdown</p>';
const markdown = await convert(html);

// With options
const markdown = await convert(html, {
  headingStyle: 'Atx',
  codeBlockStyle: 'Backticks',
  wrap: true,
});
```

**Smart backend selection:** The package automatically uses native Node.js bindings (~691k ops/sec) when available, falling back to WebAssembly (~229k ops/sec) for universal compatibility.

See [packages/html-to-markdown/README.md](packages/html-to-markdown/README.md) for full TypeScript API.

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

Comparative throughput for typical workloads:

| Implementation         | Operations/sec | Relative Speed |
| ---------------------- | -------------- | -------------- |
| **Native Node (NAPI)** | ~691,000       | **1.0×**       |
| Rust Binary/Python     | ~500-600,000   | 0.8×           |
| **WebAssembly**        | ~229,000       | 0.33×          |
| Pure JavaScript        | ~276,000       | 0.40×          |

The Rust core delivers **16-19× performance** improvements over pure Python/JS implementations, with native bindings offering maximum throughput.

## Compatibility (v1 → v2)

- V2’s Rust core sustains **150–210 MB/s** throughput; V1 averaged **≈ 2.5 MB/s** in its Python/BeautifulSoup implementation (60–80× faster).
- The Python package offers a compatibility shim in `html_to_markdown.v1_compat` (`convert_to_markdown`, `convert_to_markdown_stream`, `markdownify`). Details and keyword mappings live in [README_PYPI.md](README_PYPI.md#v1-compatibility).
- CLI flag changes, option renames, and other breaking updates are summarised in [CHANGELOG.md](CHANGELOG.md#breaking-changes).

## Community

- Chat with us on [Discord](https://discord.gg/pXxagNK2zN)
- Explore the broader [Kreuzberg](https://kreuzberg.dev) document-processing ecosystem
- Sponsor development via [GitHub Sponsors](https://github.com/sponsors/Goldziher)
