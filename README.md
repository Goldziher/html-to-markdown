# html-to-markdown

High-performance HTML → Markdown conversion powered by Rust. Shipping as a Rust crate, Python package, and standalone CLI with identical rendering behaviour.

[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://github.com/Goldziher/html-to-markdown)
[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg)](https://github.com/Goldziher/html-to-markdown)
[![Python Versions](https://img.shields.io/pypi/pyversions/html-to-markdown.svg)](https://github.com/Goldziher/html-to-markdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

## Documentation

- **Python guide** – [README_PYPI.md](README_PYPI.md)
- **Rust guide** – [crates/html-to-markdown/README.md](crates/html-to-markdown/README.md)
- **Changelog** – [CHANGELOG.md](CHANGELOG.md)
- **Contributing** – [CONTRIBUTING.md](CONTRIBUTING.md)

## Installation

> Naming: the Rust crate is published as `html-to-markdown-rs`, the Python package is `html-to-markdown`, and the CLI binary is `html-to-markdown`.

| Target                          | Command                                                                   |
| ------------------------------- | ------------------------------------------------------------------------- |
| Python package (bindings + CLI) | `pip install html-to-markdown`                                            |
| Rust crate                      | `cargo add html-to-markdown-rs`                                           |
| Rust CLI                        | `cargo install html-to-markdown-cli`                                      |
| Homebrew CLI                    | `brew tap goldziher/tap`<br>`brew install html-to-markdown`               |
| Releases                        | [GitHub Releases](https://github.com/Goldziher/html-to-markdown/releases) |

## Quick Start

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

## Compatibility (v1 → v2)

- V2’s Rust core sustains **150–210 MB/s** throughput; V1 averaged **≈ 2.5 MB/s** in its Python/BeautifulSoup implementation (60–80× faster).
- The Python package offers a compatibility shim in `html_to_markdown.v1_compat` (`convert_to_markdown`, `convert_to_markdown_stream`, `markdownify`). Details and keyword mappings live in [README_PYPI.md](README_PYPI.md#v1-compatibility).
- CLI flag changes, option renames, and other breaking updates are summarised in [CHANGELOG.md](CHANGELOG.md#breaking-changes).

## Community

- Chat with us on [Discord](https://discord.gg/pXxagNK2zN)
- Explore the broader [Kreuzberg](https://kreuzberg.dev) document-processing ecosystem
- Sponsor development via [GitHub Sponsors](https://github.com/sponsors/Goldziher)
