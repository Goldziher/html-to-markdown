# html-to-markdown

A modern, fully typed Python library for converting HTML to Markdown. This library is a completely rewritten fork
of [markdownify](https://pypi.org/project/markdownify/) with a modernized codebase, strict type safety and support for
Python 3.10+.

## Support This Project

If you find html-to-markdown useful, please consider sponsoring the development:

<a href="https://github.com/sponsors/Goldziher"><img src="https://img.shields.io/badge/Sponsor-%E2%9D%A4-pink?logo=github-sponsors" alt="Sponsor on GitHub" height="32"></a>

Your support helps maintain and improve this library for the community.

## Features

- **Full HTML5 Support**: Comprehensive support for all modern HTML5 elements including semantic, form, table, ruby, interactive, structural, SVG, and math elements
- **Table Support**: Advanced handling of complex tables with rowspan/colspan support
- **Type Safety**: Strict MyPy adherence with comprehensive type hints
- **Metadata Extraction**: Automatic extraction of document metadata (title, meta tags) as comment headers
- **Streaming Support**: Memory-efficient processing for large documents with progress callbacks
- **Highlight Support**: Multiple styles for highlighted text (`<mark>` elements)
- **Task List Support**: Converts HTML checkboxes to GitHub-compatible task list syntax
- **Flexible Configuration**: Comprehensive configuration options for customizing conversion behavior
- **CLI Tool**: Full-featured command-line interface with complete API parity
- **Custom Converters**: Extensible converter system for custom HTML tag handling
- **List Formatting**: Configurable list indentation with Discord/Slack compatibility
- **HTML Preprocessing**: Clean messy HTML with configurable aggressiveness levels
- **Bytes Input Support**: Direct handling of bytes input with automatic encoding detection and configurable source encoding
- **Whitespace Control**: Normalized or strict whitespace preservation modes
- **BeautifulSoup Integration**: Support for pre-configured BeautifulSoup instances
- **Parser Normalization**: Consistent output across all supported parsers (html.parser, lxml, html5lib)
- **Robustly Tested**: Comprehensive unit tests and integration tests covering all conversion scenarios

## Installation

```shell
pip install html-to-markdown
```

### Optional Parsers

For improved performance and compatibility, you can install with optional parsers:

```shell
# Fast lxml parser (recommended)
pip install html-to-markdown[lxml]

# Standards-compliant html5lib parser
pip install html-to-markdown[html5lib]
```

**Parser Options:**

- **html.parser** (default): Built-in Python parser, no dependencies
- **lxml**: **Recommended** - Fastest parser with good malformed HTML handling
- **html5lib**: Most standards-compliant, handles edge cases best

The library automatically uses lxml when available and **normalizes output to ensure consistent results regardless of parser choice**. We recommend using the **lxml parser for optimal performance** - it's significantly faster than the other options while maintaining excellent compatibility.

You can explicitly specify a parser using the `parser` parameter.

## Quick Start

Convert HTML to Markdown with a single function call:

```python
from html_to_markdown import convert_to_markdown

html = """
<!DOCTYPE html>
<html>
<head>
    <title>Sample Document</title>
    <meta name="description" content="A sample HTML document">
</head>
<body>
    <article>
        <h1>Welcome</h1>
        <p>This is a <strong>sample</strong> with a <a href="https://example.com">link</a>.</p>
        <p>Here's some <mark>highlighted text</mark> and a task list:</p>
        <ul>
            <li><input type="checkbox" checked> Completed task</li>
            <li><input type="checkbox"> Pending task</li>
        </ul>
    </article>
</body>
</html>
"""

markdown = convert_to_markdown(html)
print(markdown)
```

Output:

```markdown
<!--
title: Sample Document
meta-description: A sample HTML document
-->

# Welcome

This is a **sample** with a [link](https://example.com).

Here's some ==highlighted text== and a task list:

* [x] Completed task
* [ ] Pending task
```

### Working with BeautifulSoup

If you need more control over HTML parsing, you can pass a pre-configured BeautifulSoup instance:

```python
from bs4 import BeautifulSoup
from html_to_markdown import convert_to_markdown

# Configure BeautifulSoup with your preferred parser
soup = BeautifulSoup(html, "lxml")  # Note: lxml requires additional installation
markdown = convert_to_markdown(soup)
```

### Working with Bytes and Encodings

The library can directly handle bytes input, which is useful when working with HTTP responses or files:

```python
import requests
from html_to_markdown import convert_to_markdown

# Working with HTTP responses (bytes)
response = requests.get("https://example.com")
markdown = convert_to_markdown(response.content)  # response.content returns bytes

# Specify encoding for non-UTF-8 content
response = requests.get("https://example.fr")
markdown = convert_to_markdown(response.content, source_encoding="latin-1")

# Common encoding examples
html_bytes = b"<p>Hello World</p>"
markdown = convert_to_markdown(html_bytes)  # UTF-8 by default

# Latin-1 encoded content
html_latin1 = "<p>Café résumé</p>".encode("latin-1")
markdown = convert_to_markdown(html_latin1, source_encoding="latin-1")

# Windows-1252 encoded content
html_windows = '<p>Smart quotes: "Hello"</p>'.encode("windows-1252")
markdown = convert_to_markdown(html_windows, source_encoding="windows-1252")

# Piping bytes from command line
# echo '<p>Hello</p>' | python -m html_to_markdown
# cat file.html | python -m html_to_markdown --source-encoding latin-1
```

## Common Use Cases

### Discord/Slack Compatible Lists

Discord and Slack require 2-space indentation for nested lists:

**Python:**

```python
from html_to_markdown import convert_to_markdown

html = "<ul><li>Item 1<ul><li>Nested item</li></ul></li></ul>"
markdown = convert_to_markdown(html, list_indent_width=2)
# Output: * Item 1\n  + Nested item
```

**CLI:**

```shell
html_to_markdown --list-indent-width 2 input.html
```

### Cleaning Web-Scraped HTML

Remove navigation, advertisements, and forms from scraped content:

**Python:**

```python
markdown = convert_to_markdown(html, preprocess_html=True, preprocessing_preset="aggressive")
```

**CLI:**

```shell
html_to_markdown --preprocess-html --preprocessing-preset aggressive input.html
```

### Preserving Whitespace for Documentation

Maintain exact whitespace for code documentation or technical content:

**Python:**

```python
markdown = convert_to_markdown(html, whitespace_mode="strict")
```

**CLI:**

```shell
html_to_markdown --whitespace-mode strict input.html
```

### Using Tabs for List Indentation

Some editors and platforms prefer tab-based indentation:

**Python:**

```python
markdown = convert_to_markdown(html, list_indent_type="tabs")
```

**CLI:**

```shell
html_to_markdown --list-indent-type tabs input.html
```

## Advanced Usage

### Configuration Example

```python
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(
    html,
    # Headers and formatting
    heading_style="atx",
    strong_em_symbol="*",
    bullets="*+-",
    highlight_style="double-equal",
    # List indentation
    list_indent_type="spaces",
    list_indent_width=4,
    # Whitespace handling
    whitespace_mode="normalized",
    # HTML preprocessing
    preprocess_html=True,
    preprocessing_preset="standard",
)
```

### Custom Converters

Custom converters allow you to override the default conversion behavior for any HTML tag. This is particularly useful for customizing header formatting or implementing domain-specific conversion rules.

#### Basic Example: Custom Header Formatting

```python
from bs4.element import Tag
from html_to_markdown import convert_to_markdown

def custom_h1_converter(*, tag: Tag, text: str, **kwargs) -> str:
    """Convert h1 tags with custom formatting."""
    return f"### {text.upper()} ###\n\n"

def custom_h2_converter(*, tag: Tag, text: str, **kwargs) -> str:
    """Convert h2 tags with underline."""
    return f"{text}\n{'=' * len(text)}\n\n"

html = "<h1>Title</h1><h2>Subtitle</h2><p>Content</p>"
markdown = convert_to_markdown(html, custom_converters={"h1": custom_h1_converter, "h2": custom_h2_converter})
print(markdown)
# Output:
# ### TITLE ###
#
# Subtitle
# ========
#
# Content
```

#### Advanced Example: Context-Aware Link Conversion

```python
def smart_link_converter(*, tag: Tag, text: str, **kwargs) -> str:
    """Convert links based on their attributes."""
    href = tag.get("href", "")
    title = tag.get("title", "")

    # Handle different link types
    if href.startswith("http"):
        # External link
        return f"[{text}]({href} \"{title or 'External link'}\")"
    elif href.startswith("#"):
        # Anchor link
        return f"[{text}]({href})"
    elif href.startswith("mailto:"):
        # Email link
        return f"[{text}]({href})"
    else:
        # Relative link
        return f"[{text}]({href})"

html = '<a href="https://example.com">External</a> <a href="#section">Anchor</a>'
markdown = convert_to_markdown(html, custom_converters={"a": smart_link_converter})
```

#### Converter Function Signature

All converter functions must follow this signature:

```python
def converter(*, tag: Tag, text: str, **kwargs) -> str:
    """
    Args:
        tag: BeautifulSoup Tag object with access to all HTML attributes
        text: Pre-processed text content of the tag
        **kwargs: Additional context passed through from conversion

    Returns:
        Markdown formatted string
    """
    pass
```

Custom converters take precedence over built-in converters and can be used alongside other configuration options.

### Streaming API

For processing large documents with memory constraints, use the streaming API:

```python
from html_to_markdown import convert_to_markdown_stream

# Process large HTML in chunks
with open("large_document.html", "r") as f:
    html_content = f.read()

# Returns a generator that yields markdown chunks
for chunk in convert_to_markdown_stream(html_content, chunk_size=2048):
    print(chunk, end="")
```

With progress tracking:

```python
def show_progress(processed: int, total: int):
    if total > 0:
        percent = (processed / total) * 100
        print(f"\rProgress: {percent:.1f}%", end="")

# Stream with progress callback
markdown = convert_to_markdown(html_content, stream_processing=True, chunk_size=4096, progress_callback=show_progress)
```

#### When to Use Streaming vs Regular Processing

Based on comprehensive performance analysis, here are our recommendations:

**📄 Use Regular Processing When:**

- Files < 100KB (simplicity preferred)
- Simple scripts and one-off conversions
- Memory is not a concern
- You want the simplest API

**🌊 Use Streaming Processing When:**

- Files > 100KB (memory efficiency)
- Processing many files in batch
- Memory is constrained
- You need progress reporting
- You want to process results incrementally
- Running in production environments

**📋 Specific Recommendations by File Size:**

| File Size  | Recommendation                                  | Reason                                 |
| ---------- | ----------------------------------------------- | -------------------------------------- |
| < 50KB     | Regular (simplicity) or Streaming (3-5% faster) | Either works well                      |
| 50KB-100KB | Either (streaming slightly preferred)           | Minimal difference                     |
| 100KB-1MB  | Streaming preferred                             | Better performance + memory efficiency |
| > 1MB      | Streaming strongly recommended                  | Significant memory advantages          |

**🔧 Configuration Recommendations:**

- **Default chunk_size: 2048 bytes** (optimal performance balance)
- **For very large files (>10MB)**: Consider `chunk_size=4096`
- **For memory-constrained environments**: Use smaller chunks `chunk_size=1024`

**📈 Performance Benefits:**

Streaming provides consistent **3-5% performance improvement** across all file sizes:

- **Streaming throughput**: ~0.47-0.48 MB/s
- **Regular throughput**: ~0.44-0.47 MB/s
- **Memory usage**: Streaming uses less peak memory for large files
- **Latency**: Streaming allows processing results before completion

### Preprocessing API

The library provides functions for preprocessing HTML before conversion, useful for cleaning messy or complex HTML:

```python
from html_to_markdown import preprocess_html, create_preprocessor

# Direct preprocessing with custom options
cleaned_html = preprocess_html(
    raw_html,
    remove_navigation=True,
    remove_forms=True,
    remove_scripts=True,
    remove_styles=True,
    remove_comments=True,
    preserve_semantic_structure=True,
    preserve_tables=True,
    preserve_media=True,
)
markdown = convert_to_markdown(cleaned_html)

# Create a preprocessor configuration from presets
config = create_preprocessor(preset="aggressive", preserve_tables=False)  # or "minimal", "standard"  # Override preset settings
markdown = convert_to_markdown(html, **config)
```

### Exception Handling

The library provides specific exception classes for better error handling:

````python
from html_to_markdown import (
    convert_to_markdown,
    HtmlToMarkdownError,
    EmptyHtmlError,
    InvalidParserError,
    ConflictingOptionsError,
    MissingDependencyError
)

try:
    markdown = convert_to_markdown(html, parser='lxml')
except MissingDependencyError:
    # lxml not installed
    markdown = convert_to_markdown(html, parser='html.parser')
except EmptyHtmlError:
    print("No HTML content to convert")
except InvalidParserError as e:
    print(f"Parser error: {e}")
except ConflictingOptionsError as e:
    print(f"Conflicting options: {e}")
except HtmlToMarkdownError as e:
    print(f"Conversion error: {e}")

## CLI Usage

Convert HTML files directly from the command line with full access to all API options:

```shell
# Convert a file
html_to_markdown input.html > output.md

# Process stdin
cat input.html | html_to_markdown > output.md

# Use custom options
html_to_markdown --heading-style atx --wrap --wrap-width 100 input.html > output.md

# Discord-compatible lists with HTML preprocessing
html_to_markdown \
  --list-indent-width 2 \
  --preprocess-html \
  --preprocessing-preset aggressive \
  input.html > output.md
````

### Key CLI Options

**Most Common Options:**

```shell
--list-indent-width WIDTH           # Spaces per indent (default: 4, use 2 for Discord)
--list-indent-type {spaces,tabs}    # Indentation type (default: spaces)
--preprocess-html                   # Enable HTML cleaning for web scraping
--whitespace-mode {normalized,strict} # Whitespace handling (default: normalized)
--heading-style {atx,atx_closed,underlined} # Header style
--no-extract-metadata               # Disable metadata extraction
--br-in-tables                      # Use <br> tags for line breaks in table cells
--source-encoding ENCODING          # Override auto-detected encoding (rarely needed)
```

**File Encoding:**

The CLI automatically detects file encoding in most cases. Use `--source-encoding` only when automatic detection fails (typically on some Windows systems or with unusual encodings):

```shell
# Override auto-detection for Latin-1 encoded file
html_to_markdown --source-encoding latin-1 input.html > output.md

# Force UTF-16 encoding when auto-detection fails
html_to_markdown --source-encoding utf-16 input.html > output.md
```

**All Available Options:**
The CLI supports all Python API parameters. Use `html_to_markdown --help` to see the complete list.

## Migration from Markdownify

For existing projects using Markdownify, a compatibility layer is provided:

```python
# Old code
from markdownify import markdownify as md

# New code - works the same way
from html_to_markdown import markdownify as md
```

The `markdownify` function is an alias for `convert_to_markdown` and provides identical functionality.

**Note**: While the compatibility layer ensures existing code continues to work, new projects should use `convert_to_markdown` directly as it provides better type hints and clearer naming.

## Configuration Reference

### Most Common Parameters

- `list_indent_width` (int, default: `4`): Number of spaces per indentation level (use 2 for Discord/Slack)
- `list_indent_type` (str, default: `'spaces'`): Use `'spaces'` or `'tabs'` for list indentation
- `heading_style` (str, default: `'underlined'`): Header style (`'underlined'`, `'atx'`, `'atx_closed'`)
- `whitespace_mode` (str, default: `'normalized'`): Whitespace handling (`'normalized'` or `'strict'`)
- `preprocess_html` (bool, default: `False`): Enable HTML preprocessing to clean messy HTML
- `extract_metadata` (bool, default: `True`): Extract document metadata as comment header

### Text Formatting

- `highlight_style` (str, default: `'double-equal'`): Style for highlighted text (`'double-equal'`, `'html'`, `'bold'`)
- `strong_em_symbol` (str, default: `'*'`): Symbol for strong/emphasized text (`'*'` or `'_'`)
- `bullets` (str, default: `'*+-'`): Characters to use for bullet points in lists
- `newline_style` (str, default: `'spaces'`): Style for handling newlines (`'spaces'` or `'backslash'`)
- `sub_symbol` (str, default: `''`): Custom symbol for subscript text
- `sup_symbol` (str, default: `''`): Custom symbol for superscript text
- `br_in_tables` (bool, default: `False`): Use `<br>` tags for line breaks in table cells instead of spaces

### Parser Options

- `parser` (str, default: auto-detect): BeautifulSoup parser to use (`'lxml'`, `'html.parser'`, `'html5lib'`)
- `preprocessing_preset` (str, default: `'standard'`): Preprocessing level (`'minimal'`, `'standard'`, `'aggressive'`)
- `remove_forms` (bool, default: `True`): Remove form elements during preprocessing
- `remove_navigation` (bool, default: `True`): Remove navigation elements during preprocessing

### Document Processing

- `convert_as_inline` (bool, default: `False`): Treat content as inline elements only
- `strip_newlines` (bool, default: `False`): Remove newlines from HTML input before processing
- `convert` (list, default: `None`): List of HTML tags to convert (None = all supported tags)
- `strip` (list, default: `None`): List of HTML tags to remove from output
- `custom_converters` (dict, default: `None`): Mapping of HTML tag names to custom converter functions

### Text Escaping

- `escape_asterisks` (bool, default: `True`): Escape `*` characters to prevent unintended formatting
- `escape_underscores` (bool, default: `True`): Escape `_` characters to prevent unintended formatting
- `escape_misc` (bool, default: `True`): Escape miscellaneous characters to prevent Markdown conflicts

### Links and Media

- `autolinks` (bool, default: `True`): Automatically convert valid URLs to Markdown links
- `default_title` (bool, default: `False`): Use default titles for elements like links
- `keep_inline_images_in` (list, default: `None`): Tags where inline images should be preserved

### Code Blocks

- `code_language` (str, default: `''`): Default language identifier for fenced code blocks
- `code_language_callback` (callable, default: `None`): Function to dynamically determine code block language

### Text Wrapping

- `wrap` (bool, default: `False`): Enable text wrapping
- `wrap_width` (int, default: `80`): Width for text wrapping

### HTML Processing

- `parser` (str, default: auto-detect): BeautifulSoup parser to use (`'lxml'`, `'html.parser'`, `'html5lib'`)
- `whitespace_mode` (str, default: `'normalized'`): How to handle whitespace (`'normalized'` intelligently cleans whitespace, `'strict'` preserves original)
- `preprocess_html` (bool, default: `False`): Enable HTML preprocessing to clean messy HTML
- `preprocessing_preset` (str, default: `'standard'`): Preprocessing aggressiveness (`'minimal'` for basic cleaning, `'standard'` for balanced, `'aggressive'` for heavy cleaning)
- `remove_forms` (bool, default: `True`): Remove form elements during preprocessing
- `remove_navigation` (bool, default: `True`): Remove navigation elements during preprocessing

## Contribution

This library is open to contribution. Feel free to open issues or submit PRs. Its better to discuss issues before
submitting PRs to avoid disappointment.

### Local Development

1. Clone the repo

1. Install system dependencies (requires Python 3.9+)

1. Install the project dependencies:

    ```shell
    uv sync --all-extras --dev
    ```

1. Install pre-commit hooks:

    ```shell
    uv run pre-commit install
    ```

1. Run tests to ensure everything works:

    ```shell
    uv run pytest
    ```

1. Run code quality checks:

    ```shell
    uv run pre-commit run --all-files
    ```

1. Make your changes and submit a PR

### Development Commands

```shell
# Run tests with coverage
uv run pytest --cov=html_to_markdown --cov-report=term-missing

# Lint and format code
uv run ruff check --fix .
uv run ruff format .

# Type checking
uv run mypy

# Test CLI during development
uv run python -m html_to_markdown input.html

# Build package
uv build
```

## License

This library uses the MIT license.

## HTML5 Element Support

This library provides comprehensive support for all modern HTML5 elements:

### Semantic Elements

- `<article>`, `<aside>`, `<figcaption>`, `<figure>`, `<footer>`, `<header>`, `<hgroup>`, `<main>`, `<nav>`, `<section>`
- `<abbr>`, `<bdi>`, `<bdo>`, `<cite>`, `<data>`, `<dfn>`, `<kbd>`, `<mark>`, `<samp>`, `<small>`, `<time>`, `<var>`
- `<del>`, `<ins>` (strikethrough and insertion tracking)

### Form Elements

- `<form>`, `<fieldset>`, `<legend>`, `<label>`, `<input>`, `<textarea>`, `<select>`, `<option>`, `<optgroup>`
- `<button>`, `<datalist>`, `<output>`, `<progress>`, `<meter>`
- Task list support: `<input type="checkbox">` converts to `- [x]` / `- [ ]`

### Table Elements

- `<table>`, `<thead>`, `<tbody>`, `<tfoot>`, `<tr>`, `<th>`, `<td>`, `<caption>`
- **Merged cell support**: Handles `rowspan` and `colspan` attributes for complex table layouts
- **Smart cleanup**: Automatically handles table styling elements for clean Markdown output

### Interactive Elements

- `<details>`, `<summary>`, `<dialog>`, `<menu>`

### Ruby Annotations

- `<ruby>`, `<rb>`, `<rt>`, `<rtc>`, `<rp>` (for East Asian typography)

### Media Elements

- `<img>`, `<picture>`, `<audio>`, `<video>`, `<iframe>`
- SVG support with data URI conversion

### Math Elements

- `<math>` (MathML support)

## Command Line Interface

The library includes a full-featured CLI tool with complete API parity:

### Basic Usage

```bash
# Convert HTML file to Markdown
html-to-markdown document.html

# Convert from stdin
echo '<h1>Title</h1><p>Content</p>' | html-to-markdown

# Read HTML file with specific encoding
html-to-markdown document.html --source-encoding latin-1

# Pipe bytes with encoding specification
cat document.html | html-to-markdown --source-encoding utf-8
```

### Advanced CLI Options

```bash
# Discord/Slack compatible lists (2-space indent)
html-to-markdown file.html --list-indent-width 2

# Clean messy HTML before conversion
html-to-markdown file.html --preprocess-html --preprocessing-preset aggressive

# Custom heading style
html-to-markdown file.html --heading-style atx

# Strip specific tags
html-to-markdown file.html --strip nav aside footer

# Convert only specific tags
html-to-markdown file.html --convert h1 h2 p a strong em

# Enable streaming for large files with progress
html-to-markdown large.html --stream-processing --show-progress

# Use specific parser (lxml recommended for best performance)
html-to-markdown file.html --parser lxml
```

### Real-World CLI Examples

```bash
# Download and convert a webpage
curl -s https://example.com | html-to-markdown --preprocess-html > output.md

# Process multiple files with different encodings
for file in *.html; do
    html-to-markdown "$file" --source-encoding latin-1 > "${file%.html}.md"
done

# Convert with custom formatting for documentation
html-to-markdown docs.html \
    --heading-style atx \
    --list-indent-width 2 \
    --highlight-style bold \
    --no-extract-metadata > docs.md
```

## Differences from markdownify

html-to-markdown is a modern, completely rewritten library inspired by markdownify but with significant improvements:

### Key Advantages

| Feature                 | markdownify      | html-to-markdown                                                       |
| ----------------------- | ---------------- | ---------------------------------------------------------------------- |
| **Type Safety**         | No type hints    | Full MyPy compliance with strict typing                                |
| **Python Support**      | Python 3.6+      | Python 3.10+ with modern features                                      |
| **HTML5 Elements**      | Basic support    | Comprehensive HTML5 support (semantic, form, table, interactive, etc.) |
| **Table Handling**      | Simple tables    | Advanced rowspan/colspan support                                       |
| **Streaming**           | Memory-intensive | Memory-efficient streaming for large documents                         |
| **CLI Tool**            | Basic            | Full-featured CLI with all API options                                 |
| **Preprocessing**       | None             | Built-in HTML cleaning with configurable presets                       |
| **Metadata Extraction** | None             | Automatic title/meta extraction as comments                            |
| **Task Lists**          | None             | GitHub-compatible checkbox conversion                                  |
| **Bytes Input**         | None             | Direct bytes support with configurable encoding                        |
| **Custom Converters**   | Class-based      | Function-based with simpler API                                        |
| **Testing**             | Basic            | Comprehensive test suite with 100% coverage                            |
| **Performance**         | Standard         | Significantly faster with recommended lxml parser                      |

### API Compatibility

While inspired by markdownify, html-to-markdown uses a more modern, explicit API:

```python
# markdownify style
from markdownify import markdownify

result = markdownify(html, heading_style="atx", strip=["nav"])

# html-to-markdown style (more explicit)
from html_to_markdown import convert_to_markdown

result = convert_to_markdown(html, heading_style="atx", strip=["nav"])
```

### Migration from markdownify

Most markdownify code can be easily migrated:

```python
# Before (markdownify)
from markdownify import markdownify as md

result = md(html, heading_style="atx")

# After (html-to-markdown)
from html_to_markdown import convert_to_markdown

result = convert_to_markdown(html, heading_style="atx")
```

Key changes when migrating:

- Import path: `markdownify` → `html_to_markdown`
- Function name: `markdownify()` → `convert_to_markdown()`
- All parameter names remain the same for common options
- New parameters available for advanced features (preprocessing, streaming, etc.)

## Acknowledgments

Special thanks to the original [markdownify](https://pypi.org/project/markdownify/) project creators and contributors for the inspiration and foundation that made this modern implementation possible.
