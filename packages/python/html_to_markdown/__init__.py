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

from html_to_markdown.api import (
    InlineImage,
    InlineImageConfig,
    InlineImageWarning,
    MetadataConfig,
    OptionsHandle,
    convert,
    convert_with_handle,
    convert_with_inline_images,
    convert_with_inline_images_handle,
    convert_with_metadata,
    convert_with_metadata_handle,
    create_options_handle,
    start_profiling,
    stop_profiling,
)
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
    "InlineImage",
    "InlineImageConfig",
    "InlineImageWarning",
    "InvalidParserError",
    "MetadataConfig",
    "MissingDependencyError",
    "OptionsHandle",
    "PreprocessingOptions",
    "convert",
    "convert_to_markdown",
    "convert_with_handle",
    "convert_with_inline_images",
    "convert_with_inline_images_handle",
    "convert_with_metadata",
    "convert_with_metadata_handle",
    "create_options_handle",
    "markdownify",
    "start_profiling",
    "stop_profiling",
]

__version__ = "2.16.1"
