# html-to-markdown

High-performance HTML to Markdown converter powered by Rust with a clean Python API.

[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Python Versions](https://img.shields.io/pypi/pyversions/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ⚡ Performance

Version 2.0 features a complete Rust rewrite, delivering **10-30x performance improvements**:

| Document Type | Size  | V1 (Python) | V2 (Rust) | Speedup |
| ------------- | ----- | ----------- | --------- | ------- |
| Small HTML    | 5KB   | 12ms        | 0.8ms     | **15x** |
| Medium Docs   | 150KB | 180ms       | 8ms       | **22x** |
| Large Docs    | 800KB | 950ms       | 35ms      | **27x** |

## Features

- **🚀 Blazing Fast**: Rust-powered conversion engine with 10-30x speedup
- **🔄 100% Backward Compatible**: V1 API works seamlessly with V2 backend
- **📊 hOCR Support**: Advanced table extraction from OCR documents
- **🎯 Type Safe**: Full type hints and `.pyi` stubs for excellent IDE support
- **🔧 Flexible**: Clean V2 dataclass API + legacy V1 kwargs API
- **🌍 Cross-Platform**: Wheels for Linux, macOS, Windows (x86_64)
- **✅ Well-Tested**: 700+ tests with dual Python + Rust coverage
- **🛠️ Modern CLI**: Native Rust CLI with comprehensive options

## Installation

```bash
pip install html-to-markdown
```

**Supported Python versions**: 3.10, 3.11, 3.12, 3.13

## Quick Start

### V2 API (Recommended)

Clean, explicit configuration with dataclasses:

```python
from html_to_markdown import convert, ConversionOptions

html = """
<h1>Welcome to V2</h1>
<p>This is <strong>fast</strong> Rust-powered conversion!</p>
<ul>
    <li>10-30x faster</li>
    <li>100% compatible</li>
    <li>Better types</li>
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
# Welcome to V2

This is **fast** Rust-powered conversion!

* 10-30x faster
+ 100% compatible
- Better types
```

### V1 API (Fully Compatible)

Your existing code works without changes:

```python
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(html, heading_style="atx")
```

## Configuration

### V2 API - Dataclass Configuration

```python
from html_to_markdown import (
    convert,
    ConversionOptions,
    PreprocessingOptions,
    ParsingOptions,
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

# Parser settings
parsing = ParsingOptions(
    encoding="utf-8",
    parser="html.parser",  # "lxml" recommended for speed
)

markdown = convert(html, options, preprocessing, parsing)
```

### V1 API - Kwargs Configuration

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

### hOCR Table Extraction

Automatically extracts tables from OCR documents:

```python
from html_to_markdown import convert, ConversionOptions

options = ConversionOptions(
    hocr_extract_tables=True,
    hocr_table_column_threshold=50,  # Column detection sensitivity
    hocr_table_row_threshold_ratio=0.5,  # Row grouping threshold
)

markdown = convert(hocr_html, options)
```

## CLI Usage

### Basic Conversion

```bash
# Convert file
html-to-markdown input.html > output.md

# From stdin
cat input.html | html-to-markdown > output.md

# With options
html-to-markdown --heading-style atx --list-indent-width 2 input.html
```

### Advanced Examples

```bash
# Clean web-scraped content
html-to-markdown \
    --preprocess \
    --preset aggressive \
    --no-extract-metadata \
    scraped.html > clean.md

# Discord-compatible lists
html-to-markdown \
    --list-indent-width 2 \
    --bullets "*" \
    input.html > discord.md

# Process hOCR with table extraction
html-to-markdown \
    --hocr-extract-tables \
    --hocr-table-column-threshold 50 \
    ocr_output.hocr > document.md
```

### CLI Help

```bash
html-to-markdown --help
```

## Migration from V1

### No Changes Needed

If you're using the v1 API, your code works as-is:

```python
# This still works!
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(html, heading_style="atx")
```

### Recommended: Migrate to V2

For new code, use the V2 API for better type safety:

```python
# Before (v1)
markdown = convert_to_markdown(html, heading_style="atx", list_indent_width=2)

# After (v2)
from html_to_markdown import convert, ConversionOptions

options = ConversionOptions(heading_style="atx", list_indent_width=2)
markdown = convert(html, options)
```

### Unsupported V1 Features

The following v1 features are not available in v2:

- `code_language_callback` - Callbacks not supported in Rust backend
- `strip` - Use preprocessing instead
- `convert` - Use preprocessing instead
- `custom_converters` - Not yet implemented
- `convert_to_markdown_stream()` - Not yet implemented

These will raise `NotImplementedError` with clear error messages.

## Configuration Reference

### ConversionOptions

| Parameter               | Type     | Default          | Description                                        |
| ----------------------- | -------- | ---------------- | -------------------------------------------------- |
| `heading_style`         | str      | `"underlined"`   | Header style: "atx", "atx_closed", "underlined"    |
| `list_indent_width`     | int      | `4`              | Spaces per indent (use 2 for Discord/Slack)        |
| `list_indent_type`      | str      | `"spaces"`       | "spaces" or "tabs"                                 |
| `bullets`               | str      | `"*+-"`          | Bullet characters for lists                        |
| `strong_em_symbol`      | str      | `"*"`            | Symbol for bold/italic: "\*" or "\_"               |
| `escape_asterisks`      | bool     | `True`           | Escape * in text                                   |
| `escape_underscores`    | bool     | `True`           | Escape _ in text                                   |
| `escape_misc`           | bool     | `True`           | Escape other special chars                         |
| `code_language`         | str      | `""`             | Default language for code blocks                   |
| `autolinks`             | bool     | `True`           | Auto-detect links                                  |
| `extract_metadata`      | bool     | `True`           | Extract HTML metadata as comments                  |
| `highlight_style`       | str      | `"double-equal"` | Style for `<mark>`: "double-equal", "html", "bold" |
| `newline_style`         | str      | `"spaces"`       | "spaces" or "backslash" for `<br>`                 |
| `sub_symbol`            | str      | `""`             | Custom subscript symbol                            |
| `sup_symbol`            | str      | `""`             | Custom superscript symbol                          |
| `wrap`                  | bool     | `False`          | Enable text wrapping                               |
| `wrap_width`            | int      | `80`             | Wrap width                                         |
| `convert_as_inline`     | bool     | `False`          | Treat as inline content only                       |
| `keep_inline_images_in` | set[str] | `None`           | Tags to preserve inline images                     |

### PreprocessingOptions

| Parameter           | Type | Default      | Description                         |
| ------------------- | ---- | ------------ | ----------------------------------- |
| `enabled`           | bool | `False`      | Enable HTML preprocessing           |
| `preset`            | str  | `"standard"` | "minimal", "standard", "aggressive" |
| `remove_navigation` | bool | `True`       | Remove nav elements                 |
| `remove_forms`      | bool | `True`       | Remove form elements                |

### ParsingOptions

| Parameter  | Type | Default         | Description                               |
| ---------- | ---- | --------------- | ----------------------------------------- |
| `encoding` | str  | `"utf-8"`       | Source encoding                           |
| `parser`   | str  | `"html.parser"` | Parser: "html.parser", "lxml", "html5lib" |

## Performance Tips

1. **Use lxml parser** when available: `pip install lxml`
1. **Enable preprocessing** for web-scraped content
1. **Adjust hOCR thresholds** for your OCR quality
1. **Use V2 API** for better type checking and IDE support

## Development

### Setup

```bash
# Clone and install
git clone https://github.com/Goldziher/html-to-markdown.git
cd html-to-markdown
uv sync --all-extras

# Install pre-commit
uv run pre-commit install

# Build Rust extension
./build_rust.sh
```

### Testing

```bash
# Python tests
uv run pytest

# Rust tests
cargo test --all-features

# Full test suite with coverage
uv run pytest --cov=html_to_markdown --cov-report=term-missing
cargo llvm-cov --all-features --lcov --output-path rust-coverage.lcov
```

### Building Wheels

```bash
# Build wheels locally
pip install cibuildwheel
cibuildwheel --output-dir wheelhouse
```

## Architecture

```text
html-to-markdown/
├── crates/
│   ├── html-to-markdown/       # Core Rust library
│   ├── html-to-markdown-py/    # Python bindings (PyO3)
│   └── html-to-markdown-cli/   # Native CLI binary
├── html_to_markdown/
│   ├── api.py                  # V2 Python API
│   ├── options.py              # Configuration dataclasses
│   ├── v1_compat.py           # V1 compatibility layer
│   ├── cli_proxy.py           # CLI argument translation
│   └── _rust.pyi              # Type stubs
└── tests/                      # 700+ tests
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Original [markdownify](https://pypi.org/project/markdownify/) for inspiration
- Rust ecosystem for excellent HTML parsing libraries
- PyO3 for seamless Rust-Python integration

## Support

If you find this library useful, consider:

<a href="https://github.com/sponsors/Goldziher">
  <img src="https://img.shields.io/badge/Sponsor-%E2%9D%A4-pink?logo=github-sponsors" alt="Sponsor" height="32">
</a>

Your support helps maintain and improve this library!
