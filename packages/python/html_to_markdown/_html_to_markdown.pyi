from typing import Literal, TypedDict

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
def convert_with_inline_images(
    html: str,
    options: ConversionOptions | None = None,
    image_config: InlineImageConfig | None = None,
) -> tuple[str, list[InlineImage], list[InlineImageWarning]]: ...
def convert_with_metadata(
    html: str,
    options: ConversionOptions | None = None,
    metadata_config: MetadataConfig | None = None,
) -> tuple[str, ExtendedMetadata]: ...
def create_options_handle(options: ConversionOptions | None = None) -> ConversionOptionsHandle: ...
def convert_with_options_handle(html: str, handle: ConversionOptionsHandle) -> str: ...
