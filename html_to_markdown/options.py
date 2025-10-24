"""Configuration options for HTML to Markdown conversion.

This module provides dataclass-based configuration for the v2 API.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal


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

    encoding: str = "utf-8"
    """Character encoding expected for the HTML input."""

    autolinks: bool = True
    """Convert bare URLs to automatic links."""

    default_title: bool = False
    """Add a default title if none exists."""

    keep_inline_images_in: set[str] | None = None
    """Parent tag names where images should remain inline."""

    br_in_tables: bool = False
    """Use <br> tags for line breaks in table cells instead of spaces."""

    hocr_spatial_tables: bool = True
    """Reconstruct tables in hOCR documents using spatial heuristics."""

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

    strip_tags: set[str] | None = None
    """HTML tags to strip from output (output only text content, no markdown conversion)."""

    preserve_tags: set[str] | None = None
    """HTML tags to preserve as-is in the output (keep original HTML). Useful for complex elements like tables."""

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

    enabled: bool = True
    """Whether to enable HTML preprocessing (enabled by default for robust handling of malformed HTML)."""

    preset: Literal["minimal", "standard", "aggressive"] = "standard"
    """Preprocessing aggressiveness level."""

    remove_navigation: bool = True
    """Remove navigation elements during preprocessing."""

    remove_forms: bool = True
    """Remove form elements during preprocessing."""
