from html_to_markdown.exceptions import (
    ConflictingOptionsError,
    EmptyHtmlError,
    HtmlToMarkdownError,
    InvalidParserError,
    MissingDependencyError,
)
from html_to_markdown.processing import convert_to_markdown, convert_to_markdown_stream

# For backward compatibility and to maintain the existing API
markdownify = convert_to_markdown

__all__ = [
    "ConflictingOptionsError",
    "EmptyHtmlError",
    "HtmlToMarkdownError",
    "InvalidParserError",
    "MissingDependencyError",
    "convert_to_markdown",
    "convert_to_markdown_stream",
    "markdownify",
]
