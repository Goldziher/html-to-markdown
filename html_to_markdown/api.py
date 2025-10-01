"""New v2 functional API for HTML to Markdown conversion.

This module provides the new functional API with dataclass-based options,
replacing the keyword-argument heavy v1 API.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any

from html_to_markdown.options import (
    ConversionOptions,
    ParsingOptions,
    PreprocessingOptions,
    StreamingOptions,
)
from html_to_markdown.processing import convert_to_markdown, convert_to_markdown_stream

if TYPE_CHECKING:
    from collections.abc import Generator

    from bs4 import BeautifulSoup


def convert(
    html: str | bytes | BeautifulSoup,
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
    parsing: ParsingOptions | None = None,
) -> str:
    """Convert HTML to Markdown.

    This is the main entry point for the v2 API, using dataclass-based configuration
    instead of keyword arguments.

    Args:
        html: HTML string, bytes, or BeautifulSoup object to convert
        options: Conversion options (uses defaults if None)
        preprocessing: HTML preprocessing options (disabled if None)
        parsing: HTML parsing options (uses defaults if None)

    Returns:
        Markdown string

    Raises:
        EmptyHtmlError: If HTML input is empty
        MissingDependencyError: If required dependencies are not installed
        ConflictingOptionsError: If conflicting options are provided

    Example:
        >>> from html_to_markdown import convert, ConversionOptions
        >>> options = ConversionOptions(heading_style="atx", list_indent_width=2)
        >>> markdown = convert("<h1>Title</h1>", options)
        >>> print(markdown)
        # Title
        <BLANKLINE>
    """
    # Use defaults if not provided
    if options is None:
        options = ConversionOptions()
    if preprocessing is None:
        preprocessing = PreprocessingOptions()
    if parsing is None:
        parsing = ParsingOptions()

    # Map options to legacy kwargs for now (until we refactor the core)
    kwargs = _options_to_kwargs(options, preprocessing, parsing)

    return convert_to_markdown(html, **kwargs)


def convert_stream(
    html: str | bytes | BeautifulSoup,
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
    parsing: ParsingOptions | None = None,
    streaming: StreamingOptions | None = None,
) -> Generator[str, None, None]:
    """Convert HTML to Markdown with streaming.

    This function yields markdown chunks as they are generated, allowing for
    memory-efficient processing of large documents.

    Args:
        html: HTML string, bytes, or BeautifulSoup object to convert
        options: Conversion options (uses defaults if None)
        preprocessing: HTML preprocessing options (disabled if None)
        parsing: HTML parsing options (uses defaults if None)
        streaming: Streaming options (uses defaults if None)

    Yields:
        Markdown string chunks

    Raises:
        EmptyHtmlError: If HTML input is empty
        MissingDependencyError: If required dependencies are not installed

    Example:
        >>> from html_to_markdown import convert_stream, ConversionOptions
        >>> options = ConversionOptions(heading_style="atx")
        >>> for chunk in convert_stream("<h1>Title</h1><p>Content</p>", options):
        ...     print(chunk, end="")
        # Title
        <BLANKLINE>
        Content
        <BLANKLINE>
    """
    # Use defaults if not provided
    if options is None:
        options = ConversionOptions()
    if preprocessing is None:
        preprocessing = PreprocessingOptions()
    if parsing is None:
        parsing = ParsingOptions()
    if streaming is None:
        streaming = StreamingOptions()

    # Map options to legacy kwargs
    kwargs = _options_to_kwargs(options, preprocessing, parsing)
    kwargs["chunk_size"] = streaming.chunk_size
    kwargs["progress_callback"] = streaming.progress_callback

    yield from convert_to_markdown_stream(html, **kwargs)


def _options_to_kwargs(
    options: ConversionOptions,
    preprocessing: PreprocessingOptions,
    parsing: ParsingOptions,
) -> dict[str, Any]:
    """Map dataclass options to legacy keyword arguments.

    This is a temporary adapter function that will be removed once we refactor
    the core conversion logic to use the new options directly.

    Args:
        options: Conversion options
        preprocessing: Preprocessing options
        parsing: Parsing options

    Returns:
        Dictionary of keyword arguments for legacy API
    """
    kwargs: dict[str, Any] = {
        # Conversion options
        "heading_style": options.heading_style,
        "list_indent_type": options.list_indent_type,
        "list_indent_width": options.list_indent_width,
        "bullets": options.bullets,
        "strong_em_symbol": options.strong_em_symbol,
        "escape_asterisks": options.escape_asterisks,
        "escape_underscores": options.escape_underscores,
        "escape_misc": options.escape_misc,
        "code_language": options.code_language,
        "code_language_callback": options.code_language_callback,
        "autolinks": options.autolinks,
        "default_title": options.default_title,
        "keep_inline_images_in": options.keep_inline_images_in,
        "br_in_tables": options.br_in_tables,
        "highlight_style": options.highlight_style,
        "extract_metadata": options.extract_metadata,
        "whitespace_mode": options.whitespace_mode,
        "strip_newlines": options.strip_newlines,
        "wrap": options.wrap,
        "wrap_width": options.wrap_width,
        "convert": options.convert,
        "strip": options.strip,
        "convert_as_inline": options.convert_as_inline,
        "sub_symbol": options.sub_symbol,
        "sup_symbol": options.sup_symbol,
        "newline_style": options.newline_style,
        "custom_converters": options.custom_converters,
        # Preprocessing options
        "preprocess_html": preprocessing.enabled,
        "preprocessing_preset": preprocessing.preset,
        "remove_navigation": preprocessing.remove_navigation,
        "remove_forms": preprocessing.remove_forms,
        "excluded_navigation_classes": preprocessing.excluded_navigation_classes,
        "extra_navigation_classes": preprocessing.extra_navigation_classes,
        # Parsing options
        "source_encoding": parsing.encoding,
        "parser": parsing.parser,
    }

    return kwargs
