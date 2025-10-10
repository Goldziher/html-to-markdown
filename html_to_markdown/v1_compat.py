"""V1 API compatibility layer.

Provides backward compatibility for the v1 convert_to_markdown API
by translating v1 kwargs to v2 ConversionOptions/PreprocessingOptions/ParsingOptions.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Iterator

from html_to_markdown import ConversionOptions, ParsingOptions, PreprocessingOptions
from html_to_markdown import convert as convert_v2


def convert_to_markdown(  # noqa: D417
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
    parser: str = "html.parser",
    source_encoding: str = "utf-8",
    code_language_callback: object | None = None,
    strip: list[str] | None = None,
    convert: list[str] | None = None,
    custom_converters: dict[str, object] | None = None,
) -> str:
    """Convert HTML to Markdown (v1 API compatibility).

    This function provides backward compatibility with the v1 API by accepting
    the same kwargs and translating them to v2 ConversionOptions.

    Note: Some v1 options are not supported in v2:
    - code_language_callback: Removed in v2
    - convert: Removed in v2
    - custom_converters: Not yet implemented in v2

    Args:
        html: HTML string to convert

    Returns:
        Markdown string

    Raises:
        NotImplementedError: If unsupported v1 options are provided
    """
    if code_language_callback is not None:
        raise NotImplementedError(
            "code_language_callback was removed in v2. Use the code_language option to set a default language."
        )
    if convert is not None:
        raise NotImplementedError("convert option was removed in v2. All supported tags are converted by default.")
    if custom_converters is not None:
        raise NotImplementedError("custom_converters is not yet implemented in v2")

    # V1 behavior: if code_language is set, use fenced code blocks (backticks)
    # V2 default is indented code blocks, so we need to override
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
        hocr_extract_tables=hocr_extract_tables,
        hocr_table_column_threshold=hocr_table_column_threshold,
        hocr_table_row_threshold_ratio=hocr_table_row_threshold_ratio,
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

    parsing = ParsingOptions(
        encoding=source_encoding,
        parser=parser,
    )

    return convert_v2(html, options, preprocessing, parsing)


def convert_to_markdown_stream(  # noqa: D417
    html: str,
    *,
    chunk_size: int = 4096,
    **kwargs: object,
) -> Iterator[str]:
    """Stream HTML to Markdown conversion (v1 API).

    Note: Streaming was removed in v2.

    Args:
        html: HTML string to convert
        chunk_size: Size of chunks to yield (not used in v2)

    Raises:
        NotImplementedError: Streaming was removed in v2
    """
    raise NotImplementedError(
        "Streaming API (convert_to_markdown_stream) was removed in v2 (html5ever does not support streaming). "
        "Use convert_to_markdown() instead."
    )


markdownify = convert_to_markdown

__all__ = ["convert_to_markdown", "convert_to_markdown_stream", "markdownify"]
