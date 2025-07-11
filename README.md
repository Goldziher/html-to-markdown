# html-to-markdown

A modern, fully typed Python library for converting HTML to Markdown. This library is a completely rewritten fork
of [markdownify](https://pypi.org/project/markdownify/) with a modernized codebase, strict type safety and support for
Python 3.9+.

## Features

- **Full HTML5 Support**: Comprehensive support for all modern HTML5 elements including semantic, form, table, ruby, interactive, structural, SVG, and math elements
- **Type Safety**: Strict MyPy adherence with comprehensive type hints
- **Metadata Extraction**: Automatic extraction of document metadata (title, meta tags) as comment headers
- **Streaming Support**: Memory-efficient processing for large documents with progress callbacks
- **Highlight Support**: Multiple styles for highlighted text (`<mark>` elements)
- **Task List Support**: Converts HTML checkboxes to GitHub-compatible task list syntax
- **Flexible Configuration**: 20+ configuration options for customizing conversion behavior
- **CLI Tool**: Full-featured command-line interface with all API options exposed
- **Custom Converters**: Extensible converter system for custom HTML tag handling
- **BeautifulSoup Integration**: Support for pre-configured BeautifulSoup instances
- **Extensive Test Coverage**: 100% test coverage requirement with comprehensive test suite

## Installation

```shell
pip install html-to-markdown
```

### Optional lxml Parser

For improved performance, you can install with the optional lxml parser:

```shell
pip install html-to-markdown[lxml]
```

The lxml parser offers:

- **~30% faster HTML parsing** compared to the default html.parser
- Better handling of malformed HTML
- More robust parsing for complex documents

Once installed, lxml is automatically used by default for better performance. You can explicitly specify a parser if needed:

```python
result = convert_to_markdown(html)  # Auto-detects: uses lxml if available, otherwise html.parser
result = convert_to_markdown(html, parser="lxml")  # Force lxml (requires installation)
result = convert_to_markdown(html, parser="html.parser")  # Force built-in parser
```

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

## Advanced Usage

### Customizing Conversion Options

The library offers extensive customization through various options:

```python
from html_to_markdown import convert_to_markdown

html = "<div>Your content here...</div>"
markdown = convert_to_markdown(
    html,
    # Document processing
    extract_metadata=True,  # Extract metadata as comment header
    convert_as_inline=False,  # Treat as block-level content
    strip_newlines=False,  # Preserve original newlines
    # Formatting options
    heading_style="atx",  # Use # style headers
    strong_em_symbol="*",  # Use * for bold/italic
    bullets="*+-",  # Define bullet point characters
    highlight_style="double-equal",  # Use == for highlighted text
    # Text processing
    wrap=True,  # Enable text wrapping
    wrap_width=100,  # Set wrap width
    escape_asterisks=True,  # Escape * characters
    escape_underscores=True,  # Escape _ characters
    escape_misc=True,  # Escape other special characters
    # Code blocks
    code_language="python",  # Default code block language
    # Streaming for large documents
    stream_processing=False,  # Enable for memory efficiency
    chunk_size=1024,  # Chunk size for streaming
)
```

### Custom Converters

You can provide your own conversion functions for specific HTML tags:

```python
from bs4.element import Tag
from html_to_markdown import convert_to_markdown

# Define a custom converter for the <b> tag
def custom_bold_converter(*, tag: Tag, text: str, **kwargs) -> str:
    return f"IMPORTANT: {text}"

html = "<p>This is a <b>bold statement</b>.</p>"
markdown = convert_to_markdown(html, custom_converters={"b": custom_bold_converter})
print(markdown)
# Output: This is a IMPORTANT: bold statement.
```

Custom converters take precedence over the built-in converters and can be used alongside other configuration options.

### Key Configuration Options

| Option              | Type | Default          | Description                                                     |
| ------------------- | ---- | ---------------- | --------------------------------------------------------------- |
| `extract_metadata`  | bool | `True`           | Extract document metadata as comment header                     |
| `convert_as_inline` | bool | `False`          | Treat content as inline elements only                           |
| `heading_style`     | str  | `'underlined'`   | Header style (`'underlined'`, `'atx'`, `'atx_closed'`)          |
| `highlight_style`   | str  | `'double-equal'` | Highlight style (`'double-equal'`, `'html'`, `'bold'`)          |
| `stream_processing` | bool | `False`          | Enable streaming for large documents                            |
| `parser`            | str  | auto-detect      | BeautifulSoup parser (auto-detects `'lxml'` or `'html.parser'`) |
| `autolinks`         | bool | `True`           | Auto-convert URLs to Markdown links                             |
| `bullets`           | str  | `'*+-'`          | Characters to use for bullet points                             |
| `escape_asterisks`  | bool | `True`           | Escape * characters                                             |
| `wrap`              | bool | `False`          | Enable text wrapping                                            |
| `wrap_width`        | int  | `80`             | Text wrap width                                                 |

For a complete list of all 20+ options, see the [Configuration Reference](#configuration-reference) section below.

## CLI Usage

Convert HTML files directly from the command line with full access to all API options:

```shell
# Convert a file
html_to_markdown input.html > output.md

# Process stdin
cat input.html | html_to_markdown > output.md

# Use custom options
html_to_markdown --heading-style atx --wrap --wrap-width 100 input.html > output.md

# Advanced options
html_to_markdown \
  --no-extract-metadata \
  --convert-as-inline \
  --highlight-style html \
  --stream-processing \
  --show-progress \
  input.html > output.md
```

### Key CLI Options

```shell
# Content processing
--convert-as-inline          # Treat content as inline elements
--no-extract-metadata        # Disable metadata extraction
--strip-newlines             # Remove newlines from input

# Formatting
--heading-style {atx,atx_closed,underlined}
--highlight-style {double-equal,html,bold}
--strong-em-symbol {*,_}
--bullets CHARS              # e.g., "*+-"

# Text escaping
--no-escape-asterisks        # Disable * escaping
--no-escape-underscores      # Disable _ escaping
--no-escape-misc             # Disable misc character escaping

# Large document processing
--stream-processing          # Enable streaming mode
--chunk-size SIZE            # Set chunk size (default: 1024)
--show-progress              # Show progress for large files

# Text wrapping
--wrap                       # Enable text wrapping
--wrap-width WIDTH           # Set wrap width (default: 80)
```

View all available options:

```shell
html_to_markdown --help
```

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

Complete list of all configuration options:

### Document Processing

- `extract_metadata` (bool, default: `True`): Extract document metadata (title, meta tags) as comment header
- `convert_as_inline` (bool, default: `False`): Treat content as inline elements only (no block elements)
- `strip_newlines` (bool, default: `False`): Remove newlines from HTML input before processing
- `convert` (list, default: `None`): List of HTML tags to convert (None = all supported tags)
- `strip` (list, default: `None`): List of HTML tags to remove from output
- `custom_converters` (dict, default: `None`): Mapping of HTML tag names to custom converter functions

### Streaming Support

- `stream_processing` (bool, default: `False`): Enable streaming processing for large documents
- `chunk_size` (int, default: `1024`): Size of chunks when using streaming processing
- `chunk_callback` (callable, default: `None`): Callback function called with each processed chunk
- `progress_callback` (callable, default: `None`): Callback function called with (processed_bytes, total_bytes)

### Text Formatting

- `heading_style` (str, default: `'underlined'`): Header style (`'underlined'`, `'atx'`, `'atx_closed'`)
- `highlight_style` (str, default: `'double-equal'`): Style for highlighted text (`'double-equal'`, `'html'`, `'bold'`)
- `strong_em_symbol` (str, default: `'*'`): Symbol for strong/emphasized text (`'*'` or `'_'`)
- `bullets` (str, default: `'*+-'`): Characters to use for bullet points in lists
- `newline_style` (str, default: `'spaces'`): Style for handling newlines (`'spaces'` or `'backslash'`)
- `sub_symbol` (str, default: `''`): Custom symbol for subscript text
- `sup_symbol` (str, default: `''`): Custom symbol for superscript text

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

## Performance

The library is optimized for performance with several key features:

- **Efficient ancestor caching**: Reduces repeated DOM traversals using context-aware caching
- **Streaming support**: Process large documents in chunks to minimize memory usage
- **Optional lxml parser**: ~30% faster parsing for complex HTML documents
- **Optimized string operations**: Minimizes string concatenations in hot paths

Typical throughput: ~2 MB/s for regular processing on modern hardware.

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

- `<table>`, `<thead>`, `<tbody>`, `<tfoot>`, `<tr>`, `<th>`, `<td>`, `<caption>`, `<col>`, `<colgroup>`

### Interactive Elements

- `<details>`, `<summary>`, `<dialog>`, `<menu>`

### Ruby Annotations

- `<ruby>`, `<rb>`, `<rt>`, `<rtc>`, `<rp>` (for East Asian typography)

### Media Elements

- `<img>`, `<picture>`, `<audio>`, `<video>`, `<iframe>`
- SVG support with data URI conversion

### Math Elements

- `<math>` (MathML support)

## Breaking Changes (Major Version)

This version introduces several breaking changes for improved consistency and functionality:

1. **Enhanced Metadata Extraction**: Now enabled by default with comprehensive extraction of title, meta tags, and link relations
1. **Improved Newline Handling**: Better normalization of excessive newlines (max 2 consecutive)
1. **Extended HTML5 Support**: Added support for 40+ new HTML5 elements
1. **Streaming API**: New streaming parameters for large document processing
1. **Task List Support**: Automatic conversion of HTML checkboxes to GitHub-compatible task lists
1. **Highlight Styles**: New `highlight_style` parameter with multiple options for `<mark>` elements

## Acknowledgments

Special thanks to the original [markdownify](https://pypi.org/project/markdownify/) project creators and contributors.
