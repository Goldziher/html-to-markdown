"""New v2 functional API for HTML to Markdown conversion.

This module provides the new functional API with dataclass-based options,
using the Rust backend for conversion.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Literal, TypedDict, cast

import html_to_markdown._html_to_markdown as _rust  # type: ignore[import-not-found]
from html_to_markdown.options import ConversionOptions, PreprocessingOptions

if TYPE_CHECKING:
    from html_to_markdown._html_to_markdown import InlineImageConfig
else:
    InlineImageConfig = _rust.InlineImageConfig  # type: ignore[misc, assignment]


class InlineImage(TypedDict):
    """Inline image extracted during conversion."""

    data: bytes
    format: str
    filename: str | None
    description: str | None
    dimensions: tuple[int, int] | None
    source: Literal["img_data_uri", "svg_element"]
    attributes: dict[str, str]


class InlineImageWarning(TypedDict):
    """Warning produced during inline image extraction."""

    index: int
    message: str


def _to_rust_preprocessing(options: PreprocessingOptions) -> _rust.PreprocessingOptions:
    """Convert high-level preprocessing options to the Rust bindings."""
    return _rust.PreprocessingOptions(
        enabled=options.enabled,
        preset=options.preset,
        remove_navigation=options.remove_navigation,
        remove_forms=options.remove_forms,
    )


def _to_rust_options(
    options: ConversionOptions,
    preprocessing: PreprocessingOptions,
) -> _rust.ConversionOptions:
    """Convert high-level conversion options to the Rust bindings."""
    return _rust.ConversionOptions(
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
        preprocessing=_to_rust_preprocessing(preprocessing),
        encoding=options.encoding,
        debug=options.debug,
        strip_tags=list(options.strip_tags) if options.strip_tags else [],
    )


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

    rust_options = _to_rust_options(options, preprocessing)
    return cast("str", _rust.convert(html, rust_options))


def convert_with_inline_images(
    html: str,
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
    image_config: InlineImageConfig | None = None,
) -> tuple[str, list[InlineImage], list[InlineImageWarning]]:
    """Convert HTML and extract inline images.

    Returns Markdown along with extracted inline images and any warnings.
    """
    if options is None:
        options = ConversionOptions()
    if preprocessing is None:
        preprocessing = PreprocessingOptions()
    if image_config is None:
        image_config = InlineImageConfig()

    rust_options = _to_rust_options(options, preprocessing)
    markdown, images, warnings = cast(
        "tuple[str, list[InlineImage], list[InlineImageWarning]]",
        _rust.convert_with_inline_images(html, rust_options, image_config),
    )
    return markdown, list(images), list(warnings)


__all__ = [
    "InlineImage",
    "InlineImageConfig",
    "InlineImageWarning",
    "convert",
    "convert_with_inline_images",
]
