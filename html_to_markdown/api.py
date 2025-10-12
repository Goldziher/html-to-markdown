"""New v2 functional API for HTML to Markdown conversion.

This module provides the new functional API with dataclass-based options,
using the Rust backend for conversion.
"""

from __future__ import annotations

import html_to_markdown._html_to_markdown as _rust  # type: ignore[import-not-found]
from html_to_markdown.options import ConversionOptions, PreprocessingOptions


def convert(
    html: str,
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
) -> str:
    """Convert HTML to Markdown using the Rust backend.

    Args:
        html: HTML string to convert.
        options: Conversion configuration options (defaults to ConversionOptions()).
        preprocessing: HTML preprocessing options (defaults to PreprocessingOptions()).

    Returns:
        Converted Markdown string.
    """
    if options is None:
        options = ConversionOptions()
    if preprocessing is None:
        preprocessing = PreprocessingOptions()

    rust_preprocessing = _rust.PreprocessingOptions(
        enabled=preprocessing.enabled,
        preset=preprocessing.preset,
        remove_navigation=preprocessing.remove_navigation,
        remove_forms=preprocessing.remove_forms,
    )

    rust_options = _rust.ConversionOptions(
        heading_style=options.heading_style,
        list_indent_type=options.list_indent_type,
        list_indent_width=options.list_indent_width,
        bullets=options.bullets,
        strong_em_symbol=options.strong_em_symbol,
        escape_asterisks=options.escape_asterisks,
        escape_underscores=options.escape_underscores,
        escape_misc=options.escape_misc,
        escape_ascii=options.escape_ascii,
        code_language=options.code_language,
        autolinks=options.autolinks,
        default_title=options.default_title,
        br_in_tables=options.br_in_tables,
        hocr_spatial_tables=options.hocr_spatial_tables,
        highlight_style=options.highlight_style,
        extract_metadata=options.extract_metadata,
        whitespace_mode=options.whitespace_mode,
        strip_newlines=options.strip_newlines,
        wrap=options.wrap,
        wrap_width=options.wrap_width,
        convert_as_inline=options.convert_as_inline,
        sub_symbol=options.sub_symbol,
        sup_symbol=options.sup_symbol,
        newline_style=options.newline_style,
        code_block_style=options.code_block_style,
        keep_inline_images_in=list(options.keep_inline_images_in) if options.keep_inline_images_in else [],
        preprocessing=rust_preprocessing,
        encoding=options.encoding,
        debug=options.debug,
        strip_tags=list(options.strip_tags) if options.strip_tags else [],
    )

    result: str = _rust.convert(html, rust_options)
    return result
