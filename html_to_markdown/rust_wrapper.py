"""Rust backend wrapper for convert_to_markdown compatibility."""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, Literal

if TYPE_CHECKING:
    from collections.abc import Callable, Iterable, Mapping

    from bs4 import BeautifulSoup

    from html_to_markdown.converters import Converter, SupportedElements

import html_to_markdown._html_to_markdown as _rust  # type: ignore[import-not-found]
from html_to_markdown.constants import ASTERISK, DOUBLE_EQUAL, SPACES, UNDERLINED, WHITESPACE_NORMALIZED


def convert_to_markdown_rust(
    source: str | bytes | BeautifulSoup,
    *,
    parser: str | None = None,
    source_encoding: str = "utf-8",
    autolinks: bool = True,
    br_in_tables: bool = False,
    bullets: str = "*+-",
    code_language: str = "",
    _code_language_callback: Callable[[Any], str] | None = None,
    _convert: str | Iterable[str] | None = None,
    convert_as_inline: bool = False,
    _custom_converters: Mapping[SupportedElements, Converter] | None = None,
    default_title: bool = False,
    escape_asterisks: bool = True,
    escape_misc: bool = True,
    escape_underscores: bool = True,
    extract_metadata: bool = True,
    heading_style: Literal["underlined", "atx", "atx_closed"] = UNDERLINED,
    highlight_style: Literal["double-equal", "html", "bold"] = DOUBLE_EQUAL,
    _keep_inline_images_in: Iterable[str] | None = None,
    list_indent_type: Literal["spaces", "tabs"] = "spaces",
    list_indent_width: int = 4,
    newline_style: Literal["spaces", "backslash"] = SPACES,
    preprocess_html: bool = False,
    preprocessing_preset: Literal["minimal", "standard", "aggressive"] = "standard",
    remove_forms: bool = True,
    remove_navigation: bool = True,
    _excluded_navigation_classes: set[str] | None = None,
    _extra_navigation_classes: set[str] | None = None,
    _strip: str | Iterable[str] | None = None,
    strip_newlines: bool = False,
    strong_em_symbol: Literal["*", "_"] = ASTERISK,
    sub_symbol: str = "",
    sup_symbol: str = "",
    whitespace_mode: Literal["normalized", "strict"] = WHITESPACE_NORMALIZED,
    wrap: bool = False,
    wrap_width: int = 80,
) -> str:
    """Convert HTML to Markdown using Rust backend.

    This is a high-performance wrapper around the Rust implementation.
    Some advanced features (custom converters, streaming) are not yet supported.
    """
    # Convert source to string
    html = source.decode(source_encoding or "utf-8", errors="replace") if isinstance(source, bytes) else str(source)

    # Strip newlines if requested
    if strip_newlines:
        html = html.replace("\n", " ").replace("\r", " ")

    # Create Rust options
    rust_preprocessing = _rust.PreprocessingOptions(
        enabled=preprocess_html,
        preset=preprocessing_preset,
        remove_navigation=remove_navigation,
        remove_forms=remove_forms,
    )

    rust_parsing = _rust.ParsingOptions(
        encoding=source_encoding,
        parser=parser or "html.parser",
    )

    rust_options = _rust.ConversionOptions(
        heading_style=heading_style,
        list_indent_type=list_indent_type,
        list_indent_width=list_indent_width,
        bullets=bullets,
        strong_em_symbol=strong_em_symbol,
        escape_asterisks=escape_asterisks,
        escape_underscores=escape_underscores,
        escape_misc=escape_misc,
        code_language=code_language,
        autolinks=autolinks,
        default_title=default_title,
        br_in_tables=br_in_tables,
        highlight_style=highlight_style,
        extract_metadata=extract_metadata,
        whitespace_mode=whitespace_mode,
        strip_newlines=strip_newlines,
        wrap=wrap,
        wrap_width=wrap_width,
        convert_as_inline=convert_as_inline,
        sub_symbol=sub_symbol,
        sup_symbol=sup_symbol,
        newline_style=newline_style,
        preprocessing=rust_preprocessing,
        parsing=rust_parsing,
    )

    result: str = _rust.convert(html, rust_options)
    return result
