# V2 API (new functional API with options)
from html_to_markdown.api import convert, convert_stream

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
    ConverterFunction,
    ParsingOptions,
    PreprocessingOptions,
    StreamingOptions,
)

# Preprocessing utilities
from html_to_markdown.preprocessor import create_preprocessor, preprocess_html

# V1 API (backward compatibility - deprecated)
from html_to_markdown.processing import convert_to_markdown, convert_to_markdown_stream

# Legacy alias
markdownify = convert_to_markdown

__all__ = [
    "ConflictingOptionsError",
    "ConversionOptions",
    "ConverterFunction",
    "EmptyHtmlError",
    "HtmlToMarkdownError",
    "InvalidParserError",
    "MissingDependencyError",
    "ParsingOptions",
    "PreprocessingOptions",
    "StreamingOptions",
    "convert",
    "convert_stream",
    "convert_to_markdown",
    "convert_to_markdown_stream",
    "create_preprocessor",
    "markdownify",
    "preprocess_html",
]
