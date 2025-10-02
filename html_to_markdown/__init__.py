# V2 API - wraps Rust implementation
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
from html_to_markdown.processing import convert_to_markdown_stream

# V1 API - Now uses Rust backend!
from html_to_markdown.rust_wrapper import convert_to_markdown_rust as convert_to_markdown

# Aliases for convenience
convert_stream = convert_to_markdown_stream
markdownify = convert_to_markdown

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
    "convert_stream",
    "convert_to_markdown",
    "convert_to_markdown_stream",
    "markdownify",
]
