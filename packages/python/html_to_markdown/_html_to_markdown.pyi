from typing import Literal, TypeAlias, TypedDict

class PreprocessingOptions:
    enabled: bool
    preset: Literal["minimal", "standard", "aggressive"]
    remove_navigation: bool
    remove_forms: bool

    def __init__(
        self,
        *,
        enabled: bool = False,
        preset: Literal["minimal", "standard", "aggressive"] = "standard",
        remove_navigation: bool = True,
        remove_forms: bool = True,
    ) -> None: ...

class ConversionOptions:
    heading_style: Literal["underlined", "atx", "atx_closed"]
    list_indent_type: Literal["spaces", "tabs"]
    list_indent_width: int
    bullets: str
    strong_em_symbol: str
    escape_asterisks: bool
    escape_underscores: bool
    escape_misc: bool
    escape_ascii: bool
    code_language: str
    autolinks: bool
    default_title: bool
    br_in_tables: bool
    hocr_spatial_tables: bool
    highlight_style: Literal["double-equal", "html", "bold", "none"]
    extract_metadata: bool
    whitespace_mode: Literal["normalized", "strict"]
    strip_newlines: bool
    wrap: bool
    wrap_width: int
    convert_as_inline: bool
    sub_symbol: str
    sup_symbol: str
    newline_style: Literal["spaces", "backslash"]
    code_block_style: Literal["indented", "backticks", "tildes"]
    keep_inline_images_in: list[str]
    preprocessing: PreprocessingOptions
    encoding: str
    debug: bool
    strip_tags: list[str]
    preserve_tags: list[str]

    def __init__(
        self,
        *,
        heading_style: Literal["underlined", "atx", "atx_closed"] = "underlined",
        list_indent_type: Literal["spaces", "tabs"] = "spaces",
        list_indent_width: int = 4,
        bullets: str = "*+-",
        strong_em_symbol: str = "*",
        escape_asterisks: bool = False,
        escape_underscores: bool = False,
        escape_misc: bool = False,
        escape_ascii: bool = False,
        code_language: str = "",
        autolinks: bool = True,
        default_title: bool = False,
        br_in_tables: bool = False,
        hocr_spatial_tables: bool = True,
        highlight_style: Literal["double-equal", "html", "bold", "none"] = "double-equal",
        extract_metadata: bool = True,
        whitespace_mode: Literal["normalized", "strict"] = "normalized",
        strip_newlines: bool = False,
        wrap: bool = False,
        wrap_width: int = 80,
        convert_as_inline: bool = False,
        sub_symbol: str = "",
        sup_symbol: str = "",
        newline_style: Literal["spaces", "backslash"] = "spaces",
        code_block_style: Literal["indented", "backticks", "tildes"] = "indented",
        keep_inline_images_in: list[str] = [],
        preprocessing: PreprocessingOptions | None = None,
        encoding: str = "utf-8",
        debug: bool = False,
        strip_tags: list[str] = [],
        preserve_tags: list[str] = [],
    ) -> None: ...

class InlineImageConfig:
    max_decoded_size_bytes: int
    filename_prefix: str | None
    capture_svg: bool
    infer_dimensions: bool

    def __init__(
        self,
        max_decoded_size_bytes: int = ...,
        filename_prefix: str | None = None,
        capture_svg: bool = True,
        infer_dimensions: bool = False,
    ) -> None: ...

class ConversionOptionsHandle:
    def __init__(self, options: ConversionOptions | None = None) -> None: ...

class InlineImage(TypedDict):
    data: bytes
    format: str
    filename: str | None
    description: str | None
    dimensions: tuple[int, int] | None
    source: Literal["img_data_uri", "svg_element"]
    attributes: dict[str, str]

class InlineImageWarning(TypedDict):
    index: int
    message: str

class MetadataConfig:
    extract_document: bool
    extract_headers: bool
    extract_links: bool
    extract_images: bool
    extract_structured_data: bool
    max_structured_data_size: int

    def __init__(
        self,
        *,
        extract_document: bool = True,
        extract_headers: bool = True,
        extract_links: bool = True,
        extract_images: bool = True,
        extract_structured_data: bool = True,
        max_structured_data_size: int = 1_000_000,
    ) -> None: ...

class DocumentMetadata(TypedDict):
    title: str | None
    description: str | None
    keywords: list[str]
    author: str | None
    canonical_url: str | None
    base_href: str | None
    language: str | None
    text_direction: str | None
    open_graph: dict[str, str]
    twitter_card: dict[str, str]
    meta_tags: dict[str, str]

class HeaderMetadata(TypedDict):
    level: int
    text: str
    id: str | None
    depth: int
    html_offset: int

class LinkMetadata(TypedDict):
    href: str
    text: str
    title: str | None
    link_type: str
    rel: list[str]
    attributes: dict[str, str]

class ImageMetadata(TypedDict):
    src: str
    alt: str | None
    title: str | None
    dimensions: tuple[int, int] | None
    image_type: str
    attributes: dict[str, str]

class StructuredData(TypedDict):
    data_type: str
    raw_json: str
    schema_type: str | None

class ExtendedMetadata(TypedDict):
    document: DocumentMetadata
    headers: list[HeaderMetadata]
    links: list[LinkMetadata]
    images: list[ImageMetadata]
    structured_data: list[StructuredData]

def convert(html: str, options: ConversionOptions | None = None) -> str: ...
def convert_json(html: str, options_json: str | None = None) -> str: ...
def convert_with_inline_images(
    html: str,
    options: ConversionOptions | None = None,
    image_config: InlineImageConfig | None = None,
) -> tuple[str, list[InlineImage], list[InlineImageWarning]]: ...
def convert_with_inline_images_handle(
    html: str,
    handle: ConversionOptionsHandle,
    image_config: InlineImageConfig | None = None,
) -> tuple[str, list[InlineImage], list[InlineImageWarning]]: ...
def convert_with_inline_images_json(
    html: str,
    options_json: str | None = None,
    image_config_json: str | None = None,
) -> tuple[str, list[InlineImage], list[InlineImageWarning]]: ...
def convert_with_metadata(
    html: str,
    options: ConversionOptions | None = None,
    metadata_config: MetadataConfig | None = None,
) -> tuple[str, ExtendedMetadata]: ...
def convert_with_metadata_handle(
    html: str,
    handle: ConversionOptionsHandle,
    metadata_config: MetadataConfig | None = None,
) -> tuple[str, ExtendedMetadata]: ...
def convert_with_metadata_json(
    html: str,
    options_json: str | None = None,
    metadata_config_json: str | None = None,
) -> tuple[str, ExtendedMetadata]: ...
def create_options_handle(options: ConversionOptions | None = None) -> ConversionOptionsHandle: ...
def create_options_handle_json(options_json: str | None = None) -> ConversionOptionsHandle: ...
def convert_with_options_handle(html: str, handle: ConversionOptionsHandle) -> str: ...

class NodeContext(TypedDict):
    node_type: str
    """Coarse-grained node type classification (e.g., 'text', 'element', 'heading')"""
    tag_name: str
    """Raw HTML tag name (e.g., 'div', 'h1', 'custom-element')"""
    attributes: dict[str, str]
    """All HTML attributes as key-value pairs"""
    depth: int
    """Depth in the DOM tree (0 = root)"""
    index_in_parent: int
    """Index among siblings (0-based)"""
    parent_tag: str | None
    """Parent element's tag name (None if root)"""
    is_inline: bool
    """Whether this element is treated as inline vs block"""

VisitResult: TypeAlias = dict[str, str]
"""Result of a visitor callback.

Allows visitors to control the conversion flow. Must be a dictionary with a 'type' key:
- {'type': 'continue'} - Continue with default conversion
- {'type': 'skip'} - Skip this element entirely
- {'type': 'preserve_html'} - Preserve original HTML
- {'type': 'custom', 'output': 'markdown'} - Replace with custom markdown
- {'type': 'error', 'message': 'error message'} - Stop with error
"""

def convert_with_visitor(
    html: str,
    options: ConversionOptions | None = None,
    visitor: object | None = None,
) -> str: ...
def convert_with_async_visitor(
    html: str,
    options: ConversionOptions | None = None,
    visitor: object | None = None,
) -> str: ...
def start_profiling(output_path: str, frequency: int | None = None) -> None: ...
def stop_profiling() -> None: ...
