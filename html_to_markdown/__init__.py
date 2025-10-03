"""html-to-markdown: Convert HTML to Markdown using Rust backend.

This package provides high-performance HTML to Markdown conversion
powered by Rust with a clean Python API.

V2 API (current):
    from html_to_markdown import convert, ConversionOptions

    options = ConversionOptions(heading_style="atx")
    markdown = convert(html, options)

V1 API (backward compatibility):
    from html_to_markdown import convert_to_markdown

    markdown = convert_to_markdown(html, heading_style="atx")
"""

# V2 API - Rust bindings
from html_to_markdown.api import convert

# Exceptions
from html_to_markdown.exceptions import (
    ConflictingOptionsError,
    EmptyHtmlError,
    HtmlToMarkdownError,
    InvalidParserError,
    MissingDependencyError,
)

# V2 Options
from html_to_markdown.options import (
    ConversionOptions,
    ParsingOptions,
    PreprocessingOptions,
)

# V1 Compatibility
from html_to_markdown.v1_compat import convert_to_markdown, convert_to_markdown_stream

__all__ = [
    "ConflictingOptionsError",
    "ConversionOptions",
    "EmptyHtmlError",
    "HtmlToMarkdownError",
    "InvalidParserError",
    "MissingDependencyError",
    "ParsingOptions",
    "PreprocessingOptions",
    "convert",
    "convert_to_markdown",
    "convert_to_markdown_stream",
]

__version__ = "2.0.0"
