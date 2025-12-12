"""V1 API compatibility layer.

Provides backward compatibility for the v1 convert_to_markdown API
by translating v1 kwargs to v2 ConversionOptions and PreprocessingOptions.
"""

from __future__ import annotations

import warnings

from html_to_markdown import ConversionOptions, PreprocessingOptions
from html_to_markdown import convert as convert_v2

DEPRECATION_MESSAGE = (
    "The v1 compatibility layer is deprecated and will be removed in v3.0. "
    "Use html_to_markdown.convert() with ConversionOptions instead."
)


def _warn_deprecated(api_name: str, *, stacklevel: int = 2) -> None:
    warnings.warn(f"{api_name} is deprecated. {DEPRECATION_MESSAGE}", DeprecationWarning, stacklevel=stacklevel)


def convert_to_markdown(
    html: str,
    *,
    heading_style: str = "underlined",
    list_indent_type: str = "spaces",
    list_indent_width: int = 4,
    bullets: str = "*+-",
    strong_em_symbol: str = "*",
    escape_asterisks: bool = True,
    escape_underscores: bool = True,
    escape_misc: bool = True,
    code_language: str = "",
    autolinks: bool = True,
    default_title: bool = False,
    br_in_tables: bool = False,
    hocr_extract_tables: bool = True,
    hocr_table_column_threshold: int = 50,
    hocr_table_row_threshold_ratio: float = 0.5,
    highlight_style: str = "double-equal",
    extract_metadata: bool = True,
    whitespace_mode: str = "normalized",
    strip_newlines: bool = False,
    wrap: bool = False,
    wrap_width: int = 80,
    convert_as_inline: bool = False,
    sub_symbol: str = "",
    sup_symbol: str = "",
    newline_style: str = "spaces",
    keep_inline_images_in: set[str] | None = None,
    preprocess: bool = False,
    preprocessing_preset: str = "standard",
    remove_navigation: bool = True,
    remove_forms: bool = True,
    source_encoding: str = "utf-8",
    code_language_callback: object | None = None,
    strip: list[str] | None = None,
    convert: list[str] | None = None,
    custom_converters: dict[str, object] | None = None,
) -> str:
    """Convert HTML to Markdown (v1 compatibility API).

    This function provides backward compatibility with the v1 API by translating
    v1-style keyword arguments to v2 ConversionOptions and PreprocessingOptions.

    Args:
        html: HTML string to convert.
        heading_style: Style for headings (default: "underlined" for v1 compatibility).
        list_indent_type: Type of indentation for lists.
        list_indent_width: Number of spaces for list indentation (v1 default: 4).
        bullets: Characters to use for unordered list bullets.
        strong_em_symbol: Symbol for strong/emphasis formatting.
        escape_asterisks: Escape asterisk characters (v1 default: True).
        escape_underscores: Escape underscore characters (v1 default: True).
        escape_misc: Escape miscellaneous Markdown characters (v1 default: True).
        code_language: Default language for code blocks.
        autolinks: Convert bare URLs to automatic links.
        default_title: Add a default title if none exists.
        br_in_tables: Use <br> tags for line breaks in table cells.
        hocr_extract_tables: Deprecated - always True in v2.
        hocr_table_column_threshold: Deprecated - uses built-in heuristics in v2.
        hocr_table_row_threshold_ratio: Deprecated - uses built-in heuristics in v2.
        highlight_style: Style for highlighting <mark> elements.
        extract_metadata: Extract metadata from HTML head.
        whitespace_mode: How to handle whitespace.
        strip_newlines: Remove newlines from HTML before processing.
        wrap: Enable text wrapping.
        wrap_width: Column width for text wrapping.
        convert_as_inline: Treat block elements as inline.
        sub_symbol: Symbol for subscript text.
        sup_symbol: Symbol for superscript text.
        newline_style: Style for newlines.
        keep_inline_images_in: Parent tag names where images should remain inline.
        preprocess: Enable HTML preprocessing.
        preprocessing_preset: Preprocessing aggressiveness level.
        remove_navigation: Remove navigation elements during preprocessing.
        remove_forms: Remove form elements during preprocessing.
        source_encoding: Character encoding expected for the HTML input.
        code_language_callback: Deprecated - not supported in v2.
        strip: HTML tags to strip from output.
        convert: Deprecated - not supported in v2.
        custom_converters: Deprecated - not yet implemented in v2.

    Returns:
        Converted Markdown string.

    Raises:
        NotImplementedError: If deprecated v1 features are used.

    .. deprecated:: 2.0
        Use :func:`html_to_markdown.convert` with :class:`ConversionOptions` instead.
        The v1 API is provided for backward compatibility only.
    """
    _warn_deprecated("convert_to_markdown()", stacklevel=2)

    if code_language_callback is not None:
        raise NotImplementedError(
            "code_language_callback was removed in v2. Use the code_language option to set a default language."
        )
    if convert is not None:
        raise NotImplementedError("convert option was removed in v2. All supported tags are converted by default.")
    if custom_converters is not None:
        raise NotImplementedError("custom_converters is not yet implemented in v2")
    if not hocr_extract_tables:
        warnings.warn(
            "hocr_extract_tables is deprecated and will be removed in a future release. "
            "Use ConversionOptions(hocr_spatial_tables=False) to disable spatial table reconstruction.",
            DeprecationWarning,
            stacklevel=2,
        )
    if hocr_table_column_threshold != 50 or hocr_table_row_threshold_ratio != 0.5:
        raise NotImplementedError(
            "hOCR table threshold overrides were removed in v2. Table reconstruction now uses built-in heuristics."
        )

    # ~keep: v1 used indented code blocks by default, but switched to backticks when a language was set
    code_block_style = "backticks" if code_language else "indented"

    options = ConversionOptions(
        heading_style=heading_style,  # type: ignore[arg-type]
        list_indent_type=list_indent_type,  # type: ignore[arg-type]
        list_indent_width=list_indent_width,
        bullets=bullets,
        strong_em_symbol=strong_em_symbol,  # type: ignore[arg-type]
        escape_asterisks=escape_asterisks,
        escape_underscores=escape_underscores,
        escape_misc=escape_misc,
        code_block_style=code_block_style,  # type: ignore[arg-type]
        code_language=code_language,
        autolinks=autolinks,
        default_title=default_title,
        br_in_tables=br_in_tables,
        hocr_spatial_tables=hocr_extract_tables,
        highlight_style=highlight_style,  # type: ignore[arg-type]
        extract_metadata=extract_metadata,
        whitespace_mode=whitespace_mode,  # type: ignore[arg-type]
        strip_newlines=strip_newlines,
        wrap=wrap,
        wrap_width=wrap_width,
        convert_as_inline=convert_as_inline,
        sub_symbol=sub_symbol,
        sup_symbol=sup_symbol,
        newline_style=newline_style,  # type: ignore[arg-type]
        keep_inline_images_in=keep_inline_images_in,
        strip_tags=set(strip) if strip else None,
    )

    preprocessing = PreprocessingOptions(
        enabled=preprocess,
        preset=preprocessing_preset,  # type: ignore[arg-type]
        remove_navigation=remove_navigation,
        remove_forms=remove_forms,
    )

    options.encoding = source_encoding
    return convert_v2(html, options, preprocessing)


def markdownify(*args: object, **kwargs: object) -> str:
    """Alias for convert_to_markdown (deprecated).

    .. deprecated:: 2.0
        Use html_to_markdown.convert() instead.
    """
    _warn_deprecated("markdownify()", stacklevel=2)
    return convert_to_markdown(*args, **kwargs)  # type: ignore[arg-type]


__all__ = ["convert_to_markdown", "markdownify"]
