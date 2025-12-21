from typing import Literal, TypedDict

class PreprocessingOptions:
    enabled: bool
    preset: Literal["minimal", "standard", "aggressive"]
    remove_navigation: bool
    remove_forms: bool

    def __init__(
        self,
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
    strong_em_symbol: Literal["*", "_"]
    escape_asterisks: bool
    escape_underscores: bool
    escape_misc: bool
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
    keep_inline_images_in: list[str]
    preprocessing: PreprocessingOptions
    encoding: str

    def __init__(
        self,
        heading_style: Literal["underlined", "atx", "atx_closed"] = "underlined",
        list_indent_type: Literal["spaces", "tabs"] = "spaces",
        list_indent_width: int = 4,
        bullets: str = "*+-",
        strong_em_symbol: Literal["*", "_"] = "*",
        escape_asterisks: bool = True,
        escape_underscores: bool = True,
        escape_misc: bool = True,
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
        keep_inline_images_in: list[str] = [],
        preprocessing: PreprocessingOptions | None = None,
        encoding: str = "utf-8",
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

def convert(html: str, options: ConversionOptions | None = None) -> str: ...
def convert_json(html: str, options_json: str | None = None) -> str: ...
def convert_with_inline_images(
    html: str,
    options: ConversionOptions | None = None,
    image_config: InlineImageConfig | None = None,
) -> tuple[str, list[InlineImage], list[InlineImageWarning]]: ...
def convert_with_inline_images_json(
    html: str,
    options_json: str | None = None,
    image_config_json: str | None = None,
) -> tuple[str, list[InlineImage], list[InlineImageWarning]]: ...
