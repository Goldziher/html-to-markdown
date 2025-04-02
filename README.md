# html-to-markdown

A modern, fully typed Python library for converting HTML to Markdown. This library is a completely rewritten fork
of [markdownify](https://pypi.org/project/markdownify/) with a modernized codebase, strict type safety and support for
Python 3.9+.

## Features

- Full type safety with strict MyPy adherence
- Functional API design
- Extensive test coverage
- Configurable conversion options
- CLI tool for easy conversions
- Support for pre-configured BeautifulSoup instances
- Strict semver versioning

## Installation

```shell
pip install html-to-markdown
```

## Quick Start

Convert HTML to Markdown with a single function call:

```python
from html_to_markdown import convert_to_markdown

html = """
<article>
    <h1>Welcome</h1>
    <p>This is a <strong>sample</strong> with a <a href="https://example.com">link</a>.</p>
    <ul>
        <li>Item 1</li>
        <li>Item 2</li>
    </ul>
</article>
"""

markdown = convert_to_markdown(html)
print(markdown)
```

Output:

```markdown
# Welcome

This is a **sample** with a [link](https://example.com).

* Item 1
* Item 2
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
    heading_style="atx",  # Use # style headers
    strong_em_symbol="*",  # Use * for bold/italic
    bullets="*+-",  # Define bullet point characters
    wrap=True,  # Enable text wrapping
    wrap_width=100,  # Set wrap width
    escape_asterisks=True,  # Escape * characters
    code_language="python",  # Default code block language
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

### Configuration Options

| Option               | Type | Default        | Description                                            |
| -------------------- | ---- | -------------- | ------------------------------------------------------ |
| `autolinks`          | bool | `True`         | Auto-convert URLs to Markdown links                    |
| `bullets`            | str  | `'*+-'`        | Characters to use for bullet points                    |
| `code_language`      | str  | `''`           | Default language for code blocks                       |
| `heading_style`      | str  | `'underlined'` | Header style (`'underlined'`, `'atx'`, `'atx_closed'`) |
| `escape_asterisks`   | bool | `True`         | Escape * characters                                    |
| `escape_underscores` | bool | `True`         | Escape _ characters                                    |
| `wrap`               | bool | `False`        | Enable text wrapping                                   |
| `wrap_width`         | int  | `80`           | Text wrap width                                        |

For a complete list of options, see the [Configuration](#configuration) section below.

## CLI Usage

Convert HTML files directly from the command line:

```shell
# Convert a file
html_to_markdown input.html > output.md

# Process stdin
cat input.html | html_to_markdown > output.md

# Use custom options
html_to_markdown --heading-style atx --wrap --wrap-width 100 input.html > output.md
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

## Configuration

Full list of configuration options:

- `autolinks`: Convert valid URLs to Markdown links automatically
- `bullets`: Characters to use for bullet points in lists
- `code_language`: Default language for fenced code blocks
- `code_language_callback`: Function to determine code block language
- `convert`: List of HTML tags to convert (None = all supported tags)
- `default_title`: Use default titles for elements like links
- `escape_asterisks`: Escape * characters
- `escape_misc`: Escape miscellaneous Markdown characters
- `escape_underscores`: Escape _ characters
- `heading_style`: Header style (underlined/atx/atx_closed)
- `keep_inline_images_in`: Tags where inline images should be kept
- `newline_style`: Style for handling newlines (spaces/backslash)
- `strip`: Tags to remove from output
- `strong_em_symbol`: Symbol for strong/emphasized text (\* or \_)
- `sub_symbol`: Symbol for subscript text
- `sup_symbol`: Symbol for superscript text
- `wrap`: Enable text wrapping
- `wrap_width`: Width for text wrapping
- `convert_as_inline`: Treat content as inline elements
- `custom_converters`: A mapping of HTML tag names to custom converter functions

## Contribution

This library is open to contribution. Feel free to open issues or submit PRs. Its better to discuss issues before
submitting PRs to avoid disappointment.

### Local Development

1. Clone the repo

1. Install the system dependencies

1. Install the full dependencies with `uv sync`

1. Install the pre-commit hooks with:

    ```shell
    pre-commit install && pre-commit install --hook-type commit-msg
    ```

1. Make your changes and submit a PR

## License

This library uses the MIT license.

## Acknowledgments

Special thanks to the original [markdownify](https://pypi.org/project/markdownify/) project creators and contributors.
