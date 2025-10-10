# html-to-markdown

High-performance HTML to Markdown converter powered by Rust with a clean Python API. Available via PyPI with pre-built wheels for all major platforms.

[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg)](https://crates.io/crates/html-to-markdown-rs)
[![Python Versions](https://img.shields.io/pypi/pyversions/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

Part of the [Kreuzberg](https://kreuzberg.dev) ecosystem for document intelligence.

## Installation

```bash
pip install html-to-markdown
```

Pre-built wheels available for:
- **Linux**: x86_64, aarch64
- **macOS**: x86_64 (Intel), arm64 (Apple Silicon)
- **Windows**: x86_64

## ⚡ Performance

Real Wikipedia documents on Apple M4:

| Document            | Size  | Latency | Throughput | Docs/sec |
| ------------------- | ----- | ------- | ---------- | -------- |
| Lists (Timeline)    | 129KB | 0.62ms  | 208 MB/s   | 1,613    |
| Tables (Countries)  | 360KB | 2.02ms  | 178 MB/s   | 495      |
| Mixed (Python wiki) | 656KB | 4.56ms  | 144 MB/s   | 219      |

**19-30x faster** than pure Python implementations.

## Quick Start

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

markdown = convert_to_markdown(html)
print(markdown)
```

Output:

```markdown
# Welcome

This is **fast** Rust-powered conversion!

* Blazing fast
* Type safe
* Easy to use
```

## Configuration

```python
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(
    html,
    heading_style="atx",  # "atx", "atx_closed", "underlined"
    list_indent_width=2,  # Discord/Slack: use 2
    bullets="*+-",  # Bullet characters
    strong_em_symbol="*",  # "*" or "_"
    escape_asterisks=True,  # Escape * in text
    code_language="python",  # Default code block language
    extract_metadata=True,  # Extract HTML metadata
)
```

### HTML Preprocessing

Clean web-scraped HTML before conversion:

```python
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(
    scraped_html,
    preprocess=True,
    preprocessing_preset="aggressive",  # "minimal", "standard", "aggressive"
)
```

## Features

- **🚀 Blazing Fast**: Pure Rust core with ultra-fast `tl` HTML parser
- **🐍 Type Safe**: Full type hints and `.pyi` stubs for excellent IDE support
- **📊 hOCR 1.2 Compliant**: Full support for all 40+ elements and 20+ properties
- **📝 CommonMark Compliant**: Follows CommonMark specification for list formatting
- **🌍 Cross-Platform**: Pre-built wheels for Linux, macOS, and Windows
- **✅ Well-Tested**: 900+ tests with dual Python + Rust coverage
- **🔧 Zero Dependencies**: No BeautifulSoup or lxml required

## hOCR 1.2 Support

Complete hOCR 1.2 specification compliance with support for all elements, properties, and metadata:

```python
from html_to_markdown import convert_to_markdown

# Option 1: Document structure extraction (NEW in v2)
# Extracts all hOCR elements and converts to structured markdown
markdown = convert_to_markdown(hocr_html)

# Option 2: Legacy table extraction (spatial reconstruction)
# Reconstructs tables from word bounding boxes
markdown = convert_to_markdown(
    hocr_html,
    hocr_extract_tables=True,
    hocr_table_column_threshold=50,
    hocr_table_row_threshold_ratio=0.5,
)
```

**Full hOCR 1.2 Spec Coverage:**
- ✅ **All 40 Element Types** - Logical structure, typesetting, floats, inline, engine-specific
- ✅ **All 20+ Properties** - bbox, baseline, textangle, poly, x_wconf, x_font, x_fsize, and more
- ✅ **All 5 Metadata Fields** - ocr-system, ocr-capabilities, ocr-number-of-pages, ocr-langs, ocr-scripts

## Configuration Reference

### ConversionOptions

| Option                           | Type  | Default       | Description                                                      |
| -------------------------------- | ----- | ------------- | ---------------------------------------------------------------- |
| `heading_style`                  | str   | `"atx"`       | Heading format: `"atx"` (#), `"atx_closed"` (# #), `"underlined"` (===) |
| `list_indent_width`              | int   | `2`           | Spaces per list indent level (CommonMark: 2)                     |
| `list_indent_type`               | str   | `"spaces"`    | `"spaces"` or `"tabs"`                                           |
| `bullets`                        | str   | `"*+-"`       | Bullet chars for unordered lists (cycles through levels)         |
| `strong_em_symbol`               | str   | `"*"`         | Symbol for bold/italic: `"*"` or `"_"`                           |
| `escape_asterisks`               | bool  | `True`        | Escape `*` in text                                               |
| `escape_underscores`             | bool  | `True`        | Escape `_` in text                                               |
| `code_language`                  | str   | `""`          | Default language for code blocks                                 |
| `code_block_style`               | str   | `"backticks"` | `"indented"` (4 spaces), `"backticks"` (\`\`\`), `"tildes"` (\~~~) |
| `extract_metadata`               | bool  | `True`        | Extract HTML metadata as comment                                 |
| `hocr_extract_tables`            | bool  | `True`        | Enable hOCR table extraction                                     |
| `hocr_table_column_threshold`    | int   | `50`          | Column detection threshold (pixels)                              |
| `hocr_table_row_threshold_ratio` | float | `0.5`         | Row grouping threshold ratio                                     |

### Preprocessing Options

| Option           | Type | Default      | Description                            |
| ---------------- | ---- | ------------ | -------------------------------------- |
| `preprocess`     | bool | `False`      | Enable HTML preprocessing              |
| `preprocessing_preset` | str | `"standard"` | `"minimal"`, `"standard"`, `"aggressive"` |

## CLI Tool

A native Rust CLI binary is also available:

```bash
# Install via pipx (recommended for CLI tools)
pipx install html-to-markdown

# Or install with pip
pip install html-to-markdown

# Use the CLI
html-to-markdown input.html > output.md
echo "<h1>Test</h1>" | html-to-markdown
```

**For Rust library usage and comprehensive documentation**, see the [GitHub repository](https://github.com/Goldziher/html-to-markdown).

## Upgrading from v1.x

All v1 code works without changes. v2 is a complete Rust rewrite with **19-30x performance improvements**:

**What Changed:**
- Complete Rust rewrite using `tl` HTML parser
- CommonMark-compliant defaults (2-space indents, minimal escaping, ATX headings)
- No BeautifulSoup or lxml dependencies

**Removed Features:**
- `code_language_callback` - use `code_language` for default language
- `strip` / `convert` options - use preprocessing instead
- `convert_to_markdown_stream()` - not supported in v2

## Links

- **GitHub Repository**: [https://github.com/Goldziher/html-to-markdown](https://github.com/Goldziher/html-to-markdown)
- **Rust Crate**: [https://crates.io/crates/html-to-markdown-rs](https://crates.io/crates/html-to-markdown-rs)
- **Discord Community**: [https://discord.gg/pXxagNK2zN](https://discord.gg/pXxagNK2zN)
- **Kreuzberg Ecosystem**: [https://kreuzberg.dev](https://kreuzberg.dev)

## License

MIT License - see [LICENSE](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE) for details.

## Support

If you find this library useful, consider [sponsoring the project](https://github.com/sponsors/Goldziher).
