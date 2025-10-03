"""html-to-markdown: Convert HTML to Markdown using Rust backend.

This package provides high-performance HTML to Markdown conversion
powered by Rust with a clean Python API.

V2 API (current):
    from html_to_markdown import convert, ConversionOptions

    options = ConversionOptions(heading_style="atx")
    markdown = convert(html, options)

V1 compatibility will be added in a future release.
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
]

__version__ = "2.0.0"
