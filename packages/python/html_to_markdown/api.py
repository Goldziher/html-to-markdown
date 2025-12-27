"""High-level Python API backed by the Rust core."""

from __future__ import annotations

import json
from dataclasses import asdict
from typing import TYPE_CHECKING, Literal, TypedDict, cast

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
    from collections.abc import Mapping

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


def _to_camel_case(name: str) -> str:
    parts = name.split("_")
    return parts[0] + "".join(part.capitalize() for part in parts[1:])


def _normalize_value(value: object) -> object:
    if isinstance(value, set):
        return list(value)
    if isinstance(value, dict):
        return {_to_camel_case(k): _normalize_value(v) for k, v in value.items() if v is not None}
    if isinstance(value, list):
        return [_normalize_value(v) for v in value]
    return value


def _normalize_payload(payload: Mapping[str, object]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in payload.items():
        if value is None:
            continue
        result[_to_camel_case(key)] = _normalize_value(value)
    return result


def _options_payload(options: ConversionOptions, preprocessing: PreprocessingOptions) -> dict[str, object]:
    payload = cast("dict[str, object]", asdict(options))
    payload["preprocessing"] = cast("dict[str, object]", asdict(preprocessing))
    return _normalize_payload(payload)


def _inline_image_config_payload(config: InlineImageConfig | dict[str, object]) -> dict[str, object]:
    if isinstance(config, dict):
        return _normalize_payload(config)
    payload = {
        "max_decoded_size_bytes": config.max_decoded_size_bytes,
        "filename_prefix": config.filename_prefix,
        "capture_svg": config.capture_svg,
        "infer_dimensions": config.infer_dimensions,
    }
    return _normalize_payload(payload)


def _metadata_config_payload(config: MetadataConfig | dict[str, object]) -> dict[str, object]:
    if isinstance(config, dict):
        return _normalize_payload(config)
    payload = {
        "extract_document": config.extract_document,
        "extract_headers": config.extract_headers,
        "extract_links": config.extract_links,
        "extract_images": config.extract_images,
        "extract_structured_data": config.extract_structured_data,
        "max_structured_data_size": config.max_structured_data_size,
    }
    return _normalize_payload(payload)


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

    payload = _options_payload(options, preprocessing)
    return _rust.convert_json(html, json.dumps(payload))


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

    payload = _options_payload(options, preprocessing)
    config_payload = _inline_image_config_payload(image_config)
    markdown, images, warnings = _rust.convert_with_inline_images_json(
        html,
        json.dumps(payload),
        json.dumps(config_payload),
    )
    return markdown, list(images), list(warnings)


def convert_with_inline_images_handle(
    html: str,
    handle: OptionsHandle,
    image_config: InlineImageConfig | None = None,
) -> tuple[str, list[InlineImage], list[InlineImageWarning]]:
    """Convert HTML and extract inline images using a pre-built options handle."""
    if image_config is None:
        image_config = InlineImageConfig()

    markdown, images, warnings = _rust.convert_with_inline_images_handle(html, handle, image_config)
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
    payload = _options_payload(options, preprocessing)
    return _rust.create_options_handle_json(json.dumps(payload))


def start_profiling(output_path: str, frequency: int | None = None) -> None:
    """Start Rust-side profiling and write a flamegraph to output_path."""
    _rust.start_profiling(output_path, frequency)


def stop_profiling() -> None:
    """Stop Rust-side profiling and flush the flamegraph."""
    _rust.stop_profiling()


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
    if options is None:
        options = ConversionOptions()
    if preprocessing is None:
        preprocessing = PreprocessingOptions()
    if metadata_config is None:
        metadata_config = MetadataConfig()

    payload = _options_payload(options, preprocessing)
    metadata_payload = _metadata_config_payload(metadata_config)
    markdown, metadata = _rust.convert_with_metadata_json(
        html,
        json.dumps(payload),
        json.dumps(metadata_payload),
    )
    return markdown, metadata


def convert_with_metadata_handle(
    html: str,
    handle: OptionsHandle,
    metadata_config: MetadataConfig | None = None,
) -> tuple[str, ExtendedMetadata]:
    """Convert HTML and extract metadata using a pre-built options handle."""
    if metadata_config is None:
        metadata_config = MetadataConfig()

    markdown, metadata = _rust.convert_with_metadata_handle(html, handle, metadata_config)
    return markdown, metadata


def convert_with_visitor(
    html: str,
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
    visitor: object | None = None,
) -> str:
    """Convert HTML with a visitor pattern.

    This function enables custom processing of HTML elements during conversion
    using a visitor object. The visitor can inspect, modify, or skip elements
    during the conversion process.

    Args:
        html: HTML string to convert
        options: Optional conversion configuration
        preprocessing: Optional preprocessing configuration
        visitor: Optional visitor object with methods like visit_text, visit_link, etc.
                 Methods should return a result dict with 'type' key:
                 - {'type': 'continue'} - Use default conversion
                 - {'type': 'skip'} - Skip this element
                 - {'type': 'preserve_html'} - Preserve as raw HTML
                 - {'type': 'custom', 'output': 'markdown'} - Use custom output
                 - {'type': 'error', 'message': 'error'} - Stop with error

    Returns:
        Converted markdown string

    Example:
        >>> class MyVisitor:
        ...     def visit_heading(self, ctx, level, text, id):
        ...         return {"type": "custom", "output": f"HEADING[{level}]: {text}"}
        >>>
        >>> visitor = MyVisitor()
        >>> markdown = convert_with_visitor("<h1>Test</h1>", visitor=visitor)
    """
    if options is None:
        options = ConversionOptions()
    if preprocessing is None:
        preprocessing = PreprocessingOptions()

    if visitor is None:
        return convert(html, options, preprocessing)

    return _rust.convert_with_visitor(html, None, visitor)


def convert_with_async_visitor(
    html: str,
    options: ConversionOptions | None = None,
    visitor: object | None = None,
) -> str:
    """Convert HTML with an async visitor pattern.

    This function enables custom processing of HTML elements during conversion
    using a visitor object with async methods. The visitor can inspect, modify,
    or skip elements during the conversion process.

    Args:
        html: HTML string to convert
        options: Optional conversion configuration
        visitor: Optional visitor object with async methods (on_element, on_text, etc.)
                 Methods should be coroutines that return a result dict with 'type' key

    Returns:
        Converted markdown string

    Example:
        >>> class MyVisitor:
        ...     async def on_element(self, context):
        ...         return {"type": "continue"}
        >>>
        >>> visitor = MyVisitor()
        >>> markdown = convert_with_async_visitor("<h1>Test</h1>", visitor=visitor)
    """
    if options is None:
        options = ConversionOptions()

    if visitor is None:
        return _rust.convert(html, None)

    _options_payload(options, PreprocessingOptions())
    return _rust.convert_with_async_visitor(html, None, visitor)


__all__ = [
    "InlineImage",
    "InlineImageConfig",
    "InlineImageWarning",
    "MetadataConfig",
    "OptionsHandle",
    "convert",
    "convert_with_async_visitor",
    "convert_with_handle",
    "convert_with_inline_images",
    "convert_with_inline_images_handle",
    "convert_with_metadata",
    "convert_with_metadata_handle",
    "convert_with_visitor",
    "create_options_handle",
    "start_profiling",
    "stop_profiling",
]
