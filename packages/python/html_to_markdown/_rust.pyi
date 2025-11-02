class ConversionOptions:
    heading_style: str
    list_indent_type: str
    list_indent_width: int
    bullets: str
    strong_em_symbol: str
    escape_asterisks: bool
    escape_underscores: bool
    escape_misc: bool
    code_language: str
    autolinks: bool
    default_title: bool
    br_in_tables: bool
    hocr_spatial_tables: bool
    highlight_style: str
    extract_metadata: bool
    whitespace_mode: str
    strip_newlines: bool
    wrap: bool
    wrap_width: int
    convert_as_inline: bool
    sub_symbol: str
    sup_symbol: str
    newline_style: str
    keep_inline_images_in: list[str]
    preprocessing: PreprocessingOptions
    encoding: str

    def __init__(
        self,
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
        hocr_spatial_tables: bool = True,
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
        keep_inline_images_in: list[str] | None = None,
        preprocessing: PreprocessingOptions | None = None,
        encoding: str = "utf-8",
    ) -> None: ...

class PreprocessingOptions:
    enabled: bool
    preset: str
    remove_navigation: bool
    remove_forms: bool

    def __init__(
        self,
        enabled: bool = False,
        preset: str = "standard",
        remove_navigation: bool = True,
        remove_forms: bool = True,
    ) -> None: ...

def convert(html: str, options: ConversionOptions | None = None) -> str: ...
