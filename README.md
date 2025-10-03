# html-to-markdown

High-performance HTML to Markdown converter. Rust crate with Python bindings and native CLI.

[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Crates.io](https://img.shields.io/crates/v/html-to-markdown.svg)](https://crates.io/crates/html-to-markdown)
[![Python Versions](https://img.shields.io/pypi/pyversions/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## ⚡ Performance

Built with Rust using `html5ever` and `ammonia` for exceptional performance:

| Document Type      | Size  | Conversion Time | Throughput   |
| ------------------ | ----- | --------------- | ------------ |
| Lists (Timeline)   | 129KB | 1.6ms           | 630 docs/sec |
| Tables (Countries) | 360KB | 5.4ms           | 185 docs/sec |
| Python Article     | 656KB | 10.8ms          | 93 docs/sec  |

**Real-world impact:**

- Process 360KB web pages at **185 pages/second**
- Convert large documentation (656KB) in **~11ms**
- Batch process 1000 documents in **5-11 seconds** depending on size

## Features

- **🚀 Blazing Fast**: Rust core with `html5ever` parser and `ammonia` sanitizer
- **🐍 Python Bindings**: Clean Python API via PyO3 with full type hints
- **🦀 Native CLI**: Rust CLI binary with comprehensive options
- **📊 hOCR Support**: Advanced table extraction from OCR documents
- **🎯 Type Safe**: Full type hints and `.pyi` stubs for excellent IDE support
- **🌍 Cross-Platform**: Wheels for Linux, macOS, Windows (x86_64 + ARM64)
- **✅ Well-Tested**: 700+ tests with dual Python + Rust coverage

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
        heading_style: "atx".to_string(),
        ..Default::default()
    };

    let markdown = convert(html, &options).unwrap();
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

### Unsupported v1 Features

Some v1 features are not available due to the Rust backend:

- `code_language_callback` - Callbacks not supported
- `strip` / `convert` options - Use preprocessing instead
- `custom_converters` - Not yet implemented
- `convert_to_markdown_stream()` - Not yet implemented

These raise `NotImplementedError` with clear messages.

## Performance Tips

1. **Use lxml parser** when available: `pip install lxml`
1. **Enable preprocessing** for web-scraped content
1. **Adjust hOCR thresholds** for your OCR quality
1. **Use V2 API** for better type checking and IDE support

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, testing, and contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

This library started out as a fork of [markdownify](https://pypi.org/project/markdownify/), with the goal of improving performance and adding modern Python typing.

## Support

If you find this library useful, consider:

<a href="https://github.com/sponsors/Goldziher">
  <img src="https://img.shields.io/badge/Sponsor-%E2%9D%A4-pink?logo=github-sponsors" alt="Sponsor" height="32">
</a>

Your support helps maintain and improve this library!
