# html-to-markdown

High-performance HTML to Markdown converter Rust crate and CLI with Python bindings and CLI. Available via PyPI, Homebrew, and Cargo. Cross-platform support for Linux, macOS, and Windows.

[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Crates.io](https://img.shields.io/crates/v/html-to-markdown.svg)](https://crates.io/crates/html-to-markdown)
[![Python Versions](https://img.shields.io/pypi/pyversions/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Documentation](https://img.shields.io/badge/docs-github-blue)](https://github.com/Goldziher/html-to-markdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

Part of the [Kreuzberg](https://kreuzberg.dev) ecosystem for document intelligence.

**📚 [Full V2 Documentation](crates/html-to-markdown/README.md)** - Comprehensive guide for Rust, Python, and CLI usage.

## ⚡ Benchmarks

### Throughput (Python API)

Real Wikipedia documents on Apple M1 Pro:

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

📊 **[Benchmark Results](BENCHMARK_RESULTS.md)** - Detailed Python API comparison
📈 **[Performance Analysis](PERFORMANCE.md)** - Rust core benchmarks and profiling
🔧 **[Benchmarking Guide](BENCHMARKS.md)** - How to run benchmarks
✅ **[CommonMark Compliance](COMMONMARK_COMPLIANCE.md)** - CommonMark specification compliance

## Features

- **🚀 Blazing Fast**: Pure Rust core with ultra-fast `tl` HTML parser
- **🐍 Python Bindings**: Clean Python API via PyO3 with full type hints
- **🦀 Native CLI**: Rust CLI binary with comprehensive options
- **📊 hOCR 1.2 Compliant**: Full support for all 40+ elements and 20+ properties
- **📝 CommonMark Compliant**: Follows CommonMark specification for list formatting
- **🎯 Type Safe**: Full type hints and `.pyi` stubs for excellent IDE support
- **🌍 Cross-Platform**: Wheels for Linux, macOS, Windows (x86_64 + ARM64)
- **✅ Well-Tested**: 900+ tests with dual Python + Rust coverage

## Installation

### Python Package

```bash
pip install html-to-markdown
```

### Rust Library

```bash
cargo add html-to-markdown
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

Clean, type-safe configuration with dataclasses:

```python
from html_to_markdown import convert, ConversionOptions

html = """
<h1>Welcome</h1>
<p>This is <strong>fast</strong> Rust-powered conversion!</p>
<ul>
    <li>Blazing fast</li>
    <li>Type safe</li>
    <li>Easy to use</li>
</ul>
"""

options = ConversionOptions(
    heading_style="atx",
    strong_em_symbol="*",
    bullets="*+-",
)

markdown = convert(html, options)
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

### Rust API

```rust
use html_to_markdown::{convert, ConversionOptions};

fn main() {
    let html = r#"
        <h1>Welcome</h1>
        <p>This is <strong>fast</strong> conversion!</p>
    "#;

    let options = ConversionOptions {
        heading_style: HeadingStyle::Atx,
        ..Default::default()
    };

    let markdown = convert(html, Some(options)).unwrap();
    println!("{}", markdown);
}
```

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

### Python: Dataclass Configuration

```python
from html_to_markdown import (
    convert,
    ConversionOptions,
    PreprocessingOptions,
)

# Conversion settings
options = ConversionOptions(
    heading_style="atx",  # "atx", "atx_closed", "underlined"
    list_indent_width=2,  # Discord/Slack: use 2
    bullets="*+-",  # Bullet characters
    strong_em_symbol="*",  # "*" or "_"
    escape_asterisks=True,  # Escape * in text
    code_language="python",  # Default code block language
    extract_metadata=True,  # Extract HTML metadata
    highlight_style="double-equal",  # "double-equal", "html", "bold"
)

# HTML preprocessing
preprocessing = PreprocessingOptions(
    enabled=True,
    preset="standard",  # "minimal", "standard", "aggressive"
    remove_navigation=True,
    remove_forms=True,
)

markdown = convert(html, options, preprocessing)
```

### Python: Legacy API (v1 compatibility)

For backward compatibility with existing v1 code:

```python
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(
    html,
    heading_style="atx",
    list_indent_width=2,
    preprocess=True,
    preprocessing_preset="standard",
)
```

## Common Use Cases

### Discord/Slack Compatible Lists

```python
from html_to_markdown import convert, ConversionOptions

options = ConversionOptions(list_indent_width=2)
markdown = convert(html, options)
```

### Clean Web-Scraped HTML

```python
from html_to_markdown import convert, PreprocessingOptions

preprocessing = PreprocessingOptions(
    enabled=True,
    preset="aggressive",  # Heavy cleaning
    remove_navigation=True,
    remove_forms=True,
)

markdown = convert(html, preprocessing=preprocessing)
```

### hOCR 1.2 Support

**Complete hOCR 1.2 specification compliance** with support for all elements, properties, and metadata:

```python
from html_to_markdown import convert, ConversionOptions

# Option 1: Document structure extraction (NEW in v2)
# Extracts all hOCR elements and converts to structured markdown
# Supports: paragraphs, sections, chapters, headers/footers, images, math, etc.
markdown = convert(hocr_html)

# Option 2: Legacy table extraction (spatial reconstruction)
# Reconstructs tables from word bounding boxes
options = ConversionOptions(
    hocr_extract_tables=True,
    hocr_table_column_threshold=50,
    hocr_table_row_threshold_ratio=0.5,
)
markdown = convert(hocr_html, options)
```

**Full hOCR 1.2 Spec Coverage:**

- ✅ **All 40 Element Types** - Logical structure (12), typesetting (6), float (13), inline (6), engine-specific (3)
- ✅ **All 20+ Properties** - bbox, baseline, textangle, poly, x_wconf, x_confs, x_font, x_fsize, order, cflow, cuts, x_bboxes, image, ppageno, lpageno, scan_res, and more
- ✅ **All 5 Metadata Fields** - ocr-system, ocr-capabilities, ocr-number-of-pages, ocr-langs, ocr-scripts
- ✅ **37 Tests** - Complete coverage of all elements and properties

**Semantic Markdown Conversion:**

| Element Category | Examples                        | Markdown Output                           |
| ---------------- | ------------------------------- | ----------------------------------------- |
| Headings         | `ocr_title`, `ocr_chapter`      | `# Heading`                               |
| Sections         | `ocr_section`, `ocr_subsection` | `##`, `###`                               |
| Structure        | `ocr_par`, `ocr_blockquote`     | Paragraphs, `> quotes`                    |
| Metadata         | `ocr_abstract`, `ocr_author`    | `**Abstract**`, `*Author*`                |
| Floats           | `ocr_header`, `ocr_footer`      | `*Header*`, `*Footer*`                    |
| Images           | `ocr_image`, `ocr_photo`        | `![alt](path)` with image property        |
| Math             | `ocr_math`, `ocr_display`       | `` `formula` ``, ```` ```equation``` ```` |
| Layout           | `ocr_separator`                 | `---` horizontal rule                     |
| Inline           | `ocrx_word`, `ocr_dropcap`      | Text, `**Letter**`                        |

**HTML Entity Handling:** Automatically decodes `&quot;`, `&apos;`, `&lt;`, `&gt;`, `&amp;` in title attributes for proper property parsing.

## Configuration Reference

**V2 Defaults (CommonMark-compliant):**

- `list_indent_width`: 2 (CommonMark standard)
- `bullets`: "\*+-" (cycles through `*`, `+`, `-` for nested levels)
- `escape_asterisks`: false (minimal escaping)
- `escape_underscores`: false (minimal escaping)
- `escape_misc`: false (minimal escaping)
- `newline_style`: "spaces" (CommonMark: two trailing spaces)
- `code_block_style`: "backticks" (fenced code blocks with \`\`\`, better whitespace preservation)
- `heading_style`: "atx" (CommonMark: `#`)
- `preprocessing.enabled`: false (no preprocessing by default)

For complete configuration reference, see **[Full Documentation](crates/html-to-markdown/README.md#configuration-reference)**.

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
