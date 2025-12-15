"""High-level Python API backed by the Rust core."""

from __future__ import annotations

from typing import TYPE_CHECKING, Literal, TypedDict

import html_to_markdown._html_to_markdown as _rust
from html_to_markdown._html_to_markdown import (
    ConversionOptionsHandle as OptionsHandle,
)
from html_to_markdown._html_to_markdown import (
    InlineImageConfig,
    MetadataConfig,
)
from html_to_markdown.options import ConversionOptions, PreprocessingOptions

if TYPE_CHECKING:
    from html_to_markdown._html_to_markdown import ExtendedMetadata  # pragma: no cover
else:
    ExtendedMetadata = dict[str, object]  # type: ignore[assignment]


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
        preserve_tags=list(options.preserve_tags) if options.preserve_tags else [],
    )


def convert(
    html: str,
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
) -> str:
    """Convert HTML to Markdown using the Rust backend."""
    if options is None and preprocessing is None:
        return _rust.convert(html, None)

    if options is None:
        options = ConversionOptions()
    if preprocessing is None:
        preprocessing = PreprocessingOptions()

    rust_options = _to_rust_options(options, preprocessing)
    return _rust.convert(html, rust_options)


def convert_with_inline_images(
    html: str,
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
    image_config: InlineImageConfig | None = None,
) -> tuple[str, list[InlineImage], list[InlineImageWarning]]:
    """Convert HTML and extract inline images."""
    if options is None:
        options = ConversionOptions()
    if preprocessing is None:
        preprocessing = PreprocessingOptions()
    if image_config is None:
        image_config = InlineImageConfig()

    rust_options = _to_rust_options(options, preprocessing)
    markdown, images, warnings = _rust.convert_with_inline_images(html, rust_options, image_config)
    return markdown, list(images), list(warnings)


def create_options_handle(
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
) -> OptionsHandle:
    """Create a reusable ConversionOptions handle backed by Rust."""
    if options is None:
        options = ConversionOptions()
    if preprocessing is None:
        preprocessing = PreprocessingOptions()
    rust_options = _to_rust_options(options, preprocessing)
    return _rust.create_options_handle(rust_options)


def convert_with_handle(html: str, handle: OptionsHandle) -> str:
    """Convert HTML using a pre-parsed ConversionOptions handle."""
    return _rust.convert_with_options_handle(html, handle)


def convert_with_metadata(
    html: str,
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
    metadata_config: MetadataConfig | None = None,
) -> tuple[str, ExtendedMetadata]:
    """Convert HTML and extract comprehensive metadata.

    Args:
        html: HTML string to convert
        options: Optional conversion configuration
        preprocessing: Optional preprocessing configuration
        metadata_config: Optional metadata extraction configuration

    Returns:
        Tuple of (markdown, metadata_dict) where metadata_dict contains:
        - document: Document-level metadata (title, description, lang, etc.)
        - headers: List of header elements with hierarchy
        - links: List of extracted hyperlinks with classification
        - images: List of extracted images with metadata
        - structured_data: List of JSON-LD, Microdata, or RDFa blocks
    """
    if not hasattr(_rust, "convert_with_metadata"):
        raise ImportError(
            "convert_with_metadata is missing from the native extension; this indicates a broken/partial installation."
        )

    if options is None:
        options = ConversionOptions()
    if preprocessing is None:
        preprocessing = PreprocessingOptions()
    if metadata_config is None:
        metadata_config = MetadataConfig()

    rust_options = _to_rust_options(options, preprocessing)
    markdown, metadata = _rust.convert_with_metadata(html, rust_options, metadata_config)
    return markdown, metadata


__all__ = [
    "InlineImage",
    "InlineImageConfig",
    "InlineImageWarning",
    "MetadataConfig",
    "OptionsHandle",
    "convert",
    "convert_with_handle",
    "convert_with_inline_images",
    "convert_with_metadata",
    "create_options_handle",
]
