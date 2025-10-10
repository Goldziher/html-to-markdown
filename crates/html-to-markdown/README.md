# html-to-markdown

High-performance HTML to Markdown converter built with Rust.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown.svg)](https://crates.io/crates/html-to-markdown)
[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Documentation](https://docs.rs/html-to-markdown/badge.svg)](https://docs.rs/html-to-markdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Fast, reliable HTML to Markdown conversion with full CommonMark compliance. Built with `html5ever` for correctness and `ammonia` for safe HTML preprocessing.

## Rust Library

### Installation

```toml
[dependencies]
html-to-markdown = "2.0"
```

### Basic Usage

```rust
use html_to_markdown::{convert, ConversionOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let html = r#"
        <h1>Welcome</h1>
        <p>This is <strong>fast</strong> conversion!</p>
        <ul>
            <li>Built with Rust</li>
            <li>CommonMark compliant</li>
        </ul>
    "#;

    let markdown = convert(html, None)?;
    println!("{}", markdown);
    Ok(())
}
```

### Configuration

```rust
use html_to_markdown::{
    convert, ConversionOptions, HeadingStyle, ListIndentType,
    PreprocessingOptions, PreprocessingPreset,
};

let options = ConversionOptions {
    heading_style: HeadingStyle::Atx,
    list_indent_width: 2,
    list_indent_type: ListIndentType::Spaces,
    bullets: "-".to_string(),
    strong_em_symbol: '*',
    escape_asterisks: false,
    escape_underscores: false,
    newline_style: html_to_markdown::NewlineStyle::Backslash,
    code_block_style: html_to_markdown::CodeBlockStyle::Indented,
    ..Default::default()
};

let markdown = convert(html, Some(options))?;
```

### With Preprocessing

```rust
use html_to_markdown::{convert, ConversionOptions, PreprocessingOptions};

let mut options = ConversionOptions::default();
options.preprocessing.enabled = true;
options.preprocessing.preset = html_to_markdown::PreprocessingPreset::Aggressive;
options.preprocessing.remove_navigation = true;
options.preprocessing.remove_forms = true;

let markdown = convert(scraped_html, Some(options))?;
```

### hOCR Table Extraction

```rust
use html_to_markdown::{convert, ConversionOptions};

let options = ConversionOptions {
    hocr_extract_tables: true,
    hocr_table_column_threshold: 50,
    hocr_table_row_threshold_ratio: 0.5,
    ..Default::default()
};

let markdown = convert(hocr_html, Some(options))?;
```

## Python Library

### Installation

```bash
pip install html-to-markdown
```

### V2 API (Recommended)

Clean, type-safe configuration with dataclasses:

```python
from html_to_markdown import convert, ConversionOptions, PreprocessingOptions

# Basic conversion
markdown = convert(html)

# With options
options = ConversionOptions(
    heading_style="atx",  # "atx", "atx_closed", "underlined"
    list_indent_width=2,  # CommonMark default
    bullets="-",  # Consistent bullet style
    strong_em_symbol="*",  # "*" or "_"
    escape_asterisks=False,  # Minimal escaping (CommonMark)
    escape_underscores=False,
    escape_misc=False,
    newline_style="backslash",  # "backslash" or "spaces"
    code_block_style="indented",  # "indented", "backticks", "tildes"
    extract_metadata=True,
    autolinks=True,
)

markdown = convert(html, options)
```

### Python Preprocessing

```python
from html_to_markdown import (
    convert,
    ConversionOptions,
    PreprocessingOptions,
)

preprocessing = PreprocessingOptions(
    enabled=True,
    preset="aggressive",  # "minimal", "standard", "aggressive"
    remove_navigation=True,
    remove_forms=True,
)

markdown = convert(scraped_html, preprocessing=preprocessing)
```

### Python hOCR Support

```python
from html_to_markdown import convert, ConversionOptions

options = ConversionOptions(
    hocr_extract_tables=True,
    hocr_table_column_threshold=50,
    hocr_table_row_threshold_ratio=0.5,
)

markdown = convert(hocr_html, options)
```

### V1 Compatibility API

Existing v1 code works without changes:

```python
from html_to_markdown import convert_to_markdown

# All v1 kwargs still supported
markdown = convert_to_markdown(
    html,
    heading_style="atx",
    list_indent_width=2,
    escape_asterisks=True,
    preprocess=True,
)
```

## CLI Installation

### via Cargo

```bash
cargo install html-to-markdown-cli
```

### via Homebrew (macOS/Linux)

```bash
brew tap goldziher/tap
brew install html-to-markdown
```

### via uv (Python tool installer)

```bash
# Install uv if needed
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install html-to-markdown CLI
uvx --from html-to-markdown html-to-markdown
```

### Download Binary

Download pre-built binaries from [GitHub Releases](https://github.com/Goldziher/html-to-markdown/releases).

## CLI Usage

### Basic Conversion

```bash
# From stdin
echo '<h1>Title</h1>' | html-to-markdown

# From file
html-to-markdown input.html

# To file
html-to-markdown input.html -o output.md

# From stdin to file
cat input.html | html-to-markdown > output.md
```

### Common Options

```bash
# ATX-style headings (# Heading)
html-to-markdown --heading-style atx input.html

# 2-space list indentation (CommonMark)
html-to-markdown --list-indent-width 2 input.html

# Custom bullet style
html-to-markdown --bullets '*+-' input.html

# Escape special characters
html-to-markdown --escape-asterisks --escape-underscores input.html
```

### Web Scraping

```bash
# Clean web-scraped HTML
html-to-markdown \
    --preprocess \
    --preset aggressive \
    --keep-navigation false \
    --keep-forms false \
    scraped.html
```

### Code Block Styles

```bash
# Indented code blocks (default, CommonMark)
html-to-markdown --code-block-style indented input.html

# Fenced code blocks with backticks
html-to-markdown --code-block-style backticks input.html

# With default language
html-to-markdown --code-block-style backticks --code-language python input.html
```

### Advanced Options

```bash
# Backslash line breaks (default, CommonMark)
html-to-markdown --newline-style backslash input.html

# Two-space line breaks
html-to-markdown --newline-style spaces input.html

# Custom subscript/superscript symbols
html-to-markdown --sub-symbol '~' --sup-symbol '^' input.html

# Strip specific tags (output text only)
html-to-markdown --strip-tags 'script,style' input.html

# Text wrapping
html-to-markdown --wrap --wrap-width 80 input.html
```

### Shell Completions

```bash
# Bash
html-to-markdown --generate-completion bash > html-to-markdown.bash
source html-to-markdown.bash

# Zsh
html-to-markdown --generate-completion zsh > _html-to-markdown
# Move to completion directory

# Fish
html-to-markdown --generate-completion fish > html-to-markdown.fish
# Move to completion directory
```

### Man Page

```bash
html-to-markdown --generate-man > html-to-markdown.1
man ./html-to-markdown.1
```

## Configuration Reference

### ConversionOptions

| Field                            | Type   | Default       | Description                                                      |
| -------------------------------- | ------ | ------------- | ---------------------------------------------------------------- |
| `heading_style`                  | enum   | `Atx`         | Heading format: `Atx` (#), `AtxClosed` (# #), `Underlined` (===) |
| `list_indent_width`              | u8     | `2`           | Spaces per list indent level (CommonMark: 2)                     |
| `list_indent_type`               | enum   | `Spaces`      | `Spaces` or `Tabs`                                               |
| `bullets`                        | String | `"-"`         | Bullet chars for unordered lists (cycles through levels)         |
| `strong_em_symbol`               | char   | `'*'`         | Symbol for bold/italic: `'*'` or `'_'`                           |
| `escape_asterisks`               | bool   | `false`       | Escape `*` in text (minimal escaping by default)                 |
| `escape_underscores`             | bool   | `false`       | Escape `_` in text (minimal escaping by default)                 |
| `escape_misc`                    | bool   | `false`       | Escape other Markdown special chars                              |
| `escape_ascii`                   | bool   | `false`       | Escape all ASCII punctuation                                     |
| `code_language`                  | String | `""`          | Default language for code blocks                                 |
| `code_block_style`               | enum   | `Indented`    | `Indented` (4 spaces), `Backticks` (\`\`\`), `Tildes` (\~~~)     |
| `autolinks`                      | bool   | `true`        | Convert bare URLs to `<url>`                                     |
| `default_title`                  | bool   | `false`       | Use href as link title if missing                                |
| `br_in_tables`                   | bool   | `false`       | Preserve `<br>` in table cells                                   |
| `highlight_style`                | enum   | `DoubleEqual` | `DoubleEqual` (==), `Html` (<mark>), `Bold` (\*\*), `None`       |
| `extract_metadata`               | bool   | `true`        | Extract HTML metadata as comment                                 |
| `whitespace_mode`                | enum   | `Normalized`  | `Normalized` or `Strict`                                         |
| `strip_newlines`                 | bool   | `false`       | Strip newlines from input                                        |
| `wrap`                           | bool   | `false`       | Enable text wrapping                                             |
| `wrap_width`                     | usize  | `80`          | Wrap column width                                                |
| `convert_as_inline`              | bool   | `false`       | Treat block elements as inline                                   |
| `sub_symbol`                     | String | `""`          | Custom subscript symbol                                          |
| `sup_symbol`                     | String | `""`          | Custom superscript symbol                                        |
| `newline_style`                  | enum   | `Backslash`   | `Backslash` (\\) or `Spaces` (two spaces)                        |
| `keep_inline_images_in`          | Vec    | `[]`          | Elements to keep inline images                                   |
| `strip_tags`                     | Vec    | `[]`          | Tags to strip (output text only)                                 |
| `hocr_extract_tables`            | bool   | `true`        | Enable hOCR table extraction                                     |
| `hocr_table_column_threshold`    | i32    | `50`          | Column detection threshold (pixels)                              |
| `hocr_table_row_threshold_ratio` | f64    | `0.5`         | Row grouping threshold ratio                                     |
| `debug`                          | bool   | `false`       | Enable debug output                                              |

### PreprocessingOptions

| Field               | Type | Default    | Description                            |
| ------------------- | ---- | ---------- | -------------------------------------- |
| `enabled`           | bool | `false`    | Enable HTML preprocessing              |
| `preset`            | enum | `Standard` | `Minimal`, `Standard`, `Aggressive`    |
| `remove_navigation` | bool | `true`     | Remove `<nav>` and navigation elements |
| `remove_forms`      | bool | `true`     | Remove `<form>` and form inputs        |

## V2 Changes from V1

See [CHANGELOG.md](../../CHANGELOG.md) for detailed migration guide.

### Key Differences

**V2 Defaults (CommonMark-compliant):**

- `list_indent_width`: 2 (was 4 in v1)
- `bullets`: "-" (was "\*+-" in v1)
- `escape_asterisks`: false (was true in v1)
- `escape_underscores`: false (was true in v1)
- `escape_misc`: false (was true in v1)
- `newline_style`: "backslash" (was "spaces" in v1)
- `code_block_style`: "indented" (was "backticks" in v1)
- `heading_style`: "atx" (was "underlined" in v1)
- `preprocessing.enabled`: false (was true in v1)

**Removed Features:**

- `code_language_callback` - use `code_language` for default language
- `strip` option - use `strip_tags` instead
- `convert` option - all tags converted by default
- `convert_to_markdown_stream()` - not supported by html5ever

**Not Yet Implemented:**

- `custom_converters` - planned for future release

## Performance

10-30x faster than v1 Python implementation:

| Document Type | Size  | v1 Time | v2 Time | Speedup |
| ------------- | ----- | ------- | ------- | ------- |
| Small HTML    | 5KB   | 12ms    | 0.8ms   | **15x** |
| Medium Docs   | 150KB | 180ms   | 8ms     | **22x** |
| Large Docs    | 800KB | 950ms   | 35ms    | **27x** |

## Links

- [GitHub Repository](https://github.com/Goldziher/html-to-markdown)
- [Changelog](../../CHANGELOG.md)
- [Rust Documentation](https://docs.rs/html-to-markdown)
- [Python Package (PyPI)](https://pypi.org/project/html-to-markdown/)
- [Discord Community](https://discord.gg/pXxagNK2zN)

## License

MIT License - see [LICENSE](../../LICENSE) for details.
