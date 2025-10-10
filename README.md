# html-to-markdown

High-performance HTML to Markdown converter built with Rust. Available as:

- **Rust crate** (`html-to-markdown-rs` on crates.io)
- **Python package** (`html-to-markdown` on PyPI)
- **CLI binary** (via Homebrew, Cargo, or direct download)

Cross-platform support for Linux, macOS, and Windows.

[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg)](https://crates.io/crates/html-to-markdown-rs)
[![Python Versions](https://img.shields.io/pypi/pyversions/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

Part of the [Kreuzberg](https://kreuzberg.dev) ecosystem for document intelligence.

## 📚 Documentation

- **[Python Users](README_PYPI.md)** - Python package documentation and examples
- **[Rust Users](crates/html-to-markdown/README.md)** - Rust crate documentation and API reference
- **[Contributing](CONTRIBUTING.md)** - Development setup and contribution guidelines
- **[Changelog](CHANGELOG.md)** - Version history and migration guides

## ⚡ Benchmarks

### Throughput (Python API)

Real Wikipedia documents on Apple M4:

| Document            | Size  | Latency | Throughput | Docs/sec |
| ------------------- | ----- | ------- | ---------- | -------- |
| Lists (Timeline)    | 129KB | 0.62ms  | 208 MB/s   | 1,613    |
| Tables (Countries)  | 360KB | 2.02ms  | 178 MB/s   | 495      |
| Mixed (Python wiki) | 656KB | 4.56ms  | 144 MB/s   | 219      |

**Throughput scales linearly** from 144-208 MB/s across all document sizes.

### Memory Usage

| Document Size | Memory Delta | Peak RSS | Leak Detection |
| ------------- | ------------ | -------- | -------------- |
| 10KB          | < 2 MB       | < 20 MB  | ✅ None        |
| 50KB          | < 8 MB       | < 35 MB  | ✅ None        |
| 500KB         | < 40 MB      | < 80 MB  | ✅ None        |

Memory usage is linear and stable across 50+ repeated conversions.

**V2 is 19-30x faster** than v1 Python/BeautifulSoup implementation.

## Features

- **🚀 Blazing Fast**: Pure Rust core with ultra-fast `tl` HTML parser
- **🐍 Python Bindings**: Clean Python API via PyO3 with full type hints
- **🦀 Native CLI**: Rust CLI binary with comprehensive options
- **📊 hOCR 1.2 Compliant**: Full support for all 40+ elements and 20+ properties
- **📝 CommonMark Compliant**: Follows CommonMark specification for list formatting
- **🎯 Type Safe**: Full type hints and `.pyi` stubs for excellent IDE support
- **🌍 Cross-Platform**: Wheels for Linux (x86_64, aarch64), macOS (x86_64, arm64), Windows (x86_64)
- **✅ Well-Tested**: 900+ tests with dual Python + Rust coverage

## Installation

> **📦 Package Names**: Due to a naming conflict on crates.io, the Rust crate is published as `html-to-markdown-rs`, while the Python package remains `html-to-markdown` on PyPI. The CLI binary name is `html-to-markdown` for both.

### Python Package

```bash
pip install html-to-markdown
```

### Rust Library

```bash
cargo add html-to-markdown-rs
```

### CLI Binary

#### via Homebrew (macOS/Linux)

```bash
brew tap goldziher/tap
brew install html-to-markdown
```

#### via Cargo

```bash
cargo install html-to-markdown-cli
```

#### Direct Download

Download pre-built binaries from [GitHub Releases](https://github.com/Goldziher/html-to-markdown/releases).

## Quick Start

### Python API

Simple function-based API:

```python
from html_to_markdown import convert_to_markdown

html = """
<h1>Welcome</h1>
<p>This is <strong>fast</strong> Rust-powered conversion!</p>
<ul>
    <li>Blazing fast</li>
    <li>Type safe</li>
    <li>Easy to use</li>
</ul>
"""

# Basic conversion
markdown = convert_to_markdown(html)

# With custom options
markdown = convert_to_markdown(
    html,
    heading_style="atx",
    strong_em_symbol="*",
    bullets="*+-",
)

print(markdown)
```

Output:

```markdown
# Welcome

This is **fast** Rust-powered conversion!

* Blazing fast
+ Type safe
- Easy to use
```

**For detailed Python documentation**, see [README_PYPI.md](README_PYPI.md).

### Rust API

```rust
use html_to_markdown_rs::{convert, ConversionOptions, HeadingStyle};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = r#"
        <h1>Welcome</h1>
        <p>This is <strong>fast</strong> conversion!</p>
        <ul>
            <li>Blazing fast</li>
            <li>Type safe</li>
            <li>Easy to use</li>
        </ul>
    "#;

    // Basic conversion
    let markdown = convert(html, None)?;

    // With custom options
    let options = ConversionOptions {
        heading_style: HeadingStyle::Atx,
        bullets: "*+-".to_string(),
        ..Default::default()
    };
    let markdown = convert(html, Some(options))?;

    println!("{}", markdown);
    Ok(())
}
```

**For detailed Rust documentation**, see [crates/html-to-markdown/README.md](crates/html-to-markdown/README.md).

### CLI Usage

```bash
# Convert file
html-to-markdown input.html > output.md

# From stdin
cat input.html | html-to-markdown > output.md

# With options
html-to-markdown --heading-style atx --list-indent-width 2 input.html

# Clean web-scraped content
html-to-markdown \
    --preprocess \
    --preset aggressive \
    --no-extract-metadata \
    scraped.html > clean.md
```

## Configuration

### Python Configuration

All options available as keyword arguments:

```python
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(
    html,
    # Heading options
    heading_style="atx",  # "atx", "atx_closed", "underlined"
    # List options
    list_indent_width=2,  # Discord/Slack: use 2
    bullets="*+-",  # Bullet characters (cycles through levels)
    # Text formatting
    strong_em_symbol="*",  # "*" or "_"
    escape_asterisks=True,  # Escape * in text
    escape_underscores=True,  # Escape _ in text
    # Code blocks
    code_language="python",  # Default code block language
    code_block_style="backticks",  # "indented", "backticks", "tildes"
    # HTML preprocessing
    preprocess=True,  # Enable HTML cleaning
    preprocessing_preset="standard",  # "minimal", "standard", "aggressive"
    # Metadata
    extract_metadata=True,  # Extract HTML metadata
)
```

### Rust Configuration

```rust
use html_to_markdown_rs::{
    convert, ConversionOptions, HeadingStyle,
    CodeBlockStyle, PreprocessingPreset
};

let options = ConversionOptions {
    // Heading options
    heading_style: HeadingStyle::Atx,

    // List options
    list_indent_width: 2,
    bullets: "*+-".to_string(),

    // Text formatting
    strong_em_symbol: '*',
    escape_asterisks: false,
    escape_underscores: false,

    // Code blocks
    code_block_style: CodeBlockStyle::Backticks,
    code_language: "python".to_string(),

    // HTML preprocessing
    preprocessing: html_to_markdown_rs::PreprocessingOptions {
        enabled: true,
        preset: PreprocessingPreset::Standard,
        ..Default::default()
    },

    ..Default::default()
};

let markdown = convert(html, Some(options))?;
```

## Common Use Cases

### Discord/Slack Compatible Lists

```python
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(html, list_indent_width=2)
```

### Clean Web-Scraped HTML

```python
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(
    scraped_html,
    preprocess=True,
    preprocessing_preset="aggressive",
)
```

### hOCR 1.2 Support

Complete hOCR 1.2 specification compliance:

```python
from html_to_markdown import convert_to_markdown

# Basic hOCR conversion (document structure)
markdown = convert_to_markdown(hocr_html)

# With table extraction from bounding boxes
markdown = convert_to_markdown(
    hocr_html,
    hocr_extract_tables=True,
    hocr_table_column_threshold=50,
)
```

**hOCR Features:**

- ✅ All 40 element types (logical structure, typesetting, floats, inline, engine-specific)
- ✅ All 20+ properties (bbox, baseline, textangle, poly, confidence scores, fonts, etc.)
- ✅ All 5 metadata fields (system, capabilities, languages, scripts, page count)
- ✅ Semantic markdown conversion (headings, sections, quotes, images, math, etc.)

**For complete hOCR documentation**, see [README_PYPI.md](README_PYPI.md).

## Configuration Reference

### ConversionOptions

| Option                           | Type  | Default          | Description                                                             |
| -------------------------------- | ----- | ---------------- | ----------------------------------------------------------------------- |
| `heading_style`                  | str   | `"atx"`          | Heading format: `"atx"` (#), `"atx_closed"` (# #), `"underlined"` (===) |
| `list_indent_width`              | int   | `2`              | Spaces per list indent level (CommonMark: 2)                            |
| `list_indent_type`               | str   | `"spaces"`       | `"spaces"` or `"tabs"`                                                  |
| `bullets`                        | str   | `"*+-"`          | Bullet chars for unordered lists (cycles through levels)                |
| `strong_em_symbol`               | str   | `"*"`            | Symbol for bold/italic: `"*"` or `"_"`                                  |
| `escape_asterisks`               | bool  | `True`           | Escape `*` in text                                                      |
| `escape_underscores`             | bool  | `True`           | Escape `_` in text                                                      |
| `escape_misc`                    | bool  | `False`          | Escape other Markdown special chars                                     |
| `code_language`                  | str   | `""`             | Default language for code blocks                                        |
| `code_block_style`               | str   | `"backticks"`    | `"indented"` (4 spaces), `"backticks"` (\`\`\`), `"tildes"` (\~~~)      |
| `highlight_style`                | str   | `"double-equal"` | `"double-equal"` (==), `"html"` (<mark>), `"bold"` (\*\*), `"none"`     |
| `extract_metadata`               | bool  | `True`           | Extract HTML metadata as comment                                        |
| `hocr_extract_tables`            | bool  | `True`           | Enable hOCR table extraction                                            |
| `hocr_table_column_threshold`    | int   | `50`             | Column detection threshold (pixels)                                     |
| `hocr_table_row_threshold_ratio` | float | `0.5`            | Row grouping threshold ratio                                            |

### PreprocessingOptions

| Option              | Type | Default      | Description                               |
| ------------------- | ---- | ------------ | ----------------------------------------- |
| `enabled`           | bool | `False`      | Enable HTML preprocessing                 |
| `preset`            | str  | `"standard"` | `"minimal"`, `"standard"`, `"aggressive"` |
| `remove_navigation` | bool | `True`       | Remove `<nav>` and navigation elements    |
| `remove_forms`      | bool | `True`       | Remove `<form>` and form inputs           |

### CLI Options

All Python options are available as CLI flags. Use `html-to-markdown --help` for full reference.

**Common CLI flags:**

- `--heading-style <STYLE>`: atx, atx-closed, underlined
- `--list-indent-width <N>`: Number of spaces for list indentation
- `--bullets <CHARS>`: Bullet characters (e.g., `*+-`)
- `--code-language <LANG>`: Default language for code blocks
- `--preprocess`: Enable HTML preprocessing
- `--preset <PRESET>`: Preprocessing preset (minimal, standard, aggressive)
- `-o, --output <FILE>`: Write output to file

## Upgrading from v1.x

### Backward Compatibility

Existing v1 code works without changes:

```python
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(html, heading_style="atx")  # Still works!
```

### Modern API (Recommended)

For new projects, use the dataclass-based API:

```python
from html_to_markdown import convert, ConversionOptions

options = ConversionOptions(heading_style="atx", list_indent_width=2)
markdown = convert(html, options)
```

### What Changed in v2

**Core Rewrite:**

- Complete Rust rewrite using `tl` HTML parser
- 19-30x performance improvement over v1
- CommonMark-compliant defaults (2-space indents, minimal escaping, ATX headings)
- No BeautifulSoup or lxml dependencies

**Removed Features:**

- `code_language_callback` - use `code_language` for default language
- `strip` / `convert` options - use `strip_tags` or preprocessing
- `convert_to_markdown_stream()` - not supported in v2

**Planned:**

- `custom_converters` - planned for future release

See **[CHANGELOG.md](CHANGELOG.md)** for complete v1 vs v2 comparison and migration guide.

## Kreuzberg Ecosystem

html-to-markdown is part of the [Kreuzberg](https://kreuzberg.dev) ecosystem, a comprehensive framework for document intelligence and processing. While html-to-markdown focuses on converting HTML to Markdown with maximum performance, Kreuzberg provides a complete solution for:

- **Document Extraction**: Extract text, images, and metadata from 50+ document formats
- **OCR Processing**: Multiple OCR backends (Tesseract, EasyOCR, PaddleOCR)
- **Table Extraction**: Vision-based and OCR-based table detection
- **Document Classification**: Automatic detection of contracts, forms, invoices, etc.
- **RAG Pipelines**: Integration with retrieval-augmented generation workflows

Learn more at [kreuzberg.dev](https://kreuzberg.dev) or join our [Discord community](https://discord.gg/pXxagNK2zN).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, testing, and contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

Version 1 started as a fork of [markdownify](https://pypi.org/project/markdownify/), rewritten, extended, and enhanced with better typing and features. Version 2 is a complete Rust rewrite for high performance.

## Support

If you find this library useful, consider:

<a href="https://github.com/sponsors/Goldziher">
  <img src="https://img.shields.io/badge/Sponsor-%E2%9D%A4-pink?logo=github-sponsors" alt="Sponsor" height="32">
</a>

Your support helps maintain and improve this library!
