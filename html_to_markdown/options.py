"""Configuration options for HTML to Markdown conversion.

This module provides dataclass-based configuration for the v2 API.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING, Any, Literal, Protocol

if TYPE_CHECKING:
    from collections.abc import Callable

    from bs4 import Tag


class ConverterFunction(Protocol):
    """Protocol for custom converter functions.

    Converter functions receive keyword-only arguments including the HTML tag,
    processed text content, and any conversion options needed.

    Example:
        >>> def custom_link_converter(*, tag: Tag, text: str, autolinks: bool, **kwargs: Any) -> str:
        ...     href = tag.get("href", "")
        ...     return f"[{text}]({href})"
    """

    def __call__(self, *, tag: Tag, text: str, **kwargs: Any) -> str:
        """Convert an HTML element to Markdown.

        Args:
            tag: BeautifulSoup Tag object representing the HTML element
            text: Processed text content of the element's children
            **kwargs: Additional conversion options (varies by converter)

        Returns:
            Markdown string representation of the element
        """
        ...


@dataclass
class ConversionOptions:
    """Main conversion configuration.

    This class groups all conversion-related options together, replacing
    the large number of keyword arguments in the v1 API.

    Example:
        >>> options = ConversionOptions(
        ...     heading_style="atx",
        ...     list_indent_width=2,
        ...     escape_asterisks=True,
        ... )
        >>> from html_to_markdown import convert
        >>> markdown = convert("<h1>Title</h1>", options)
    """

    heading_style: Literal["underlined", "atx", "atx_closed"] = "atx"
    """Style for headings: 'atx' (#) is CommonMark default, 'underlined' (===), or 'atx_closed' (# #)."""

    list_indent_type: Literal["spaces", "tabs"] = "spaces"
    """Type of indentation for lists."""

    list_indent_width: int = 2
    """Number of spaces for list indentation (CommonMark uses 2 spaces, ignored if list_indent_type='tabs')."""

    bullets: str = "-*+"
    """Characters to use for unordered list bullets (cycles through -, *, + for nested levels). CommonMark compliant."""

    strong_em_symbol: Literal["*", "_"] = "*"
    """Symbol for strong/emphasis formatting."""

    escape_asterisks: bool = False
    """Escape asterisk characters in text to prevent accidental formatting. Default False for minimal escaping (CommonMark)."""

    escape_underscores: bool = False
    """Escape underscore characters in text to prevent accidental formatting. Default False for minimal escaping (CommonMark)."""

    escape_misc: bool = False
    """Escape miscellaneous Markdown characters. Default False for minimal escaping (CommonMark)."""

    escape_ascii: bool = False
    """Escape all ASCII punctuation (for CommonMark spec compliance tests). Disabled by default for minimal escaping."""

    code_language: str = ""
    """Default language for code blocks."""

    code_language_callback: Callable[[Tag], str] | None = None
    """Callback to determine code language from element."""

    autolinks: bool = True
    """Convert bare URLs to automatic links."""

    default_title: bool = False
    """Add a default title if none exists."""

    keep_inline_images_in: set[str] | None = None
    """Parent tag names where images should remain inline."""

    br_in_tables: bool = False
    """Use <br> tags for line breaks in table cells instead of spaces."""

    hocr_extract_tables: bool = True
    """Enable table extraction from hOCR (HTML-based OCR) documents."""

    hocr_table_column_threshold: int = 50
    """Pixel threshold for detecting column boundaries in hOCR tables."""

    hocr_table_row_threshold_ratio: float = 0.5
    """Row height ratio threshold for detecting row boundaries in hOCR tables."""

    highlight_style: Literal["double-equal", "html", "bold"] = "double-equal"
    """Style for highlighting <mark> elements."""

    extract_metadata: bool = True
    """Extract metadata from HTML head and include as comment."""

    whitespace_mode: Literal["normalized", "strict"] = "normalized"
    """How to handle whitespace: 'normalized' or 'strict'."""

    strip_newlines: bool = False
    """Remove newlines from HTML before processing."""

    wrap: bool = False
    """Enable text wrapping."""

    wrap_width: int = 80
    """Column width for text wrapping."""

    convert: set[str] | None = None
    """HTML tags to convert to Markdown (None = all supported tags). v1 compatibility only."""

    strip_tags: set[str] | None = None
    """HTML tags to strip from output (output only text content, no markdown conversion)."""

    convert_as_inline: bool = False
    """Treat block elements as inline during conversion."""

    sub_symbol: str = ""
    """Symbol for subscript text."""

    sup_symbol: str = ""
    """Symbol for superscript text."""

    newline_style: Literal["spaces", "backslash"] = "spaces"
    """Style for newlines: 'spaces' (two trailing spaces, CommonMark default) or 'backslash' (\\). Both are equally CommonMark compliant."""

    code_block_style: Literal["indented", "backticks", "tildes"] = "backticks"
    """Style for code blocks: 'backticks' (```, better whitespace preservation), 'indented' (4 spaces), or 'tildes' (~~~). All are CommonMark compliant."""

    custom_converters: dict[str, Callable[..., str]] | None = None
    """Custom converter functions for specific HTML elements."""

    debug: bool = False
    """Enable debug mode with diagnostic warnings about unhandled elements and hOCR processing."""


@dataclass
class PreprocessingOptions:
    """HTML preprocessing configuration.

    Controls how HTML is cleaned and preprocessed before conversion.

    Example:
        >>> options = PreprocessingOptions(
        ...     enabled=True,
        ...     preset="aggressive",
        ...     remove_navigation=True,
        ... )
    """

    enabled: bool = False
    """Whether to enable HTML preprocessing (disabled by default for minimal transformation)."""

    preset: Literal["minimal", "standard", "aggressive"] = "standard"
    """Preprocessing aggressiveness level."""

    remove_navigation: bool = True
    """Remove navigation elements during preprocessing."""

    remove_forms: bool = True
    """Remove form elements during preprocessing."""

    excluded_navigation_classes: set[str] | None = None
    """Navigation class fragments to keep even when removing navigation."""

    extra_navigation_classes: set[str] | None = None
    """Additional navigation class fragments to strip beyond defaults."""


@dataclass
class ParsingOptions:
    """HTML parsing configuration.

    Example:
        >>> options = ParsingOptions(
        ...     encoding="utf-8",
        ...     detect_encoding=True,
        ... )
    """

    encoding: str = "utf-8"
    """Character encoding for decoding bytes input."""

    detect_encoding: bool = False
    """Attempt to detect encoding from HTML (not yet implemented)."""

    parser: str | None = None
    """HTML parser to use: 'html.parser', 'lxml', or 'html5lib' (None = auto)."""
