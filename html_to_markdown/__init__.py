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

from html_to_markdown.api import convert
from html_to_markdown.exceptions import (
    ConflictingOptionsError,
    EmptyHtmlError,
    HtmlToMarkdownError,
    InvalidParserError,
    MissingDependencyError,
)
from html_to_markdown.options import ConversionOptions, PreprocessingOptions
from html_to_markdown.v1_compat import convert_to_markdown, markdownify

__all__ = [
    "ConflictingOptionsError",
    "ConversionOptions",
    "EmptyHtmlError",
    "HtmlToMarkdownError",
    "InvalidParserError",
    "MissingDependencyError",
    "PreprocessingOptions",
    "convert",
    "convert_to_markdown",
    "markdownify",
]

__version__ = "2.3.0"
