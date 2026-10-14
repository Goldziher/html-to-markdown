# Python API Reference

Package name: `html-to-markdown`
Import: `from html_to_markdown import ...`
Python requirement: 3.10+

## Primary Function

```python
def convert(
    html: str,
    options: ConversionOptions | None = None,
    visitor: HtmlVisitor | None = None,
) -> ConversionResult:
    ...
```

Returns a `ConversionResult` dataclass with all extracted data in a single pass. Pass `PreprocessingOptions` via `ConversionOptions.preprocessing`.

```python
from html_to_markdown import convert, ConversionOptions, PreprocessingOptions

# Simple
result = convert("<h1>Hello</h1><p>World</p>")
print(result.content)        # "# Hello\n\nWorld\n"
print(result.tables)         # []
print(result.warnings)       # []
print(result.metadata)       # HtmlMetadata or None

# With options
html = "<h1>Hello</h1><p>World</p>"
result = convert(
    html,
    options=ConversionOptions(
        heading_style="atx",
        code_block_style="backticks",
        preprocessing=PreprocessingOptions(enabled=True, preset="aggressive"),
    ),
)
print(result.content)
```

## ConversionResult (dataclass)

```python
from dataclasses import dataclass, field
from typing import Any
from html_to_markdown import ConversionResult

@dataclass
class ConversionResult:
    content: str | None = None           # Converted markdown/djot/plain text
    document: Any | None = None          # DocumentStructure (populated when include_document_structure=True)
    metadata: Any = None                 # HtmlMetadata
    tables: list[Any] = field(default_factory=list)   # list[TableData]
    warnings: list[Any] = field(default_factory=list) # list[ProcessingWarning]
    # Note: the Python ConversionResult has no `images` field. Inline-image binary
    # extraction is exposed only in the Rust core (the `inline-images` feature).
```

## ConversionOptions (dataclass)

```python
from dataclasses import dataclass, field
from typing import Any
from html_to_markdown import ConversionOptions

@dataclass
class ConversionOptions:
    # Headings
    heading_style: str = "atx"           # "underlined" | "atx" | "atx_closed"

    # Lists
    list_indent_type: str = "spaces"     # "spaces" | "tabs"
    list_indent_width: int = 2
    bullets: str = "-*+"

    # Emphasis
    strong_em_symbol: str = "*"          # "*" or "_"

    # Escaping
    escape_asterisks: bool = False
    escape_underscores: bool = False
    escape_misc: bool = False
    escape_ascii: bool = False

    # Code
    code_language: str = ""
    code_block_style: str = "backticks"  # "indented" | "backticks" | "tildes"

    # Links
    autolinks: bool = True
    default_title: bool = False
    link_style: str = "inline"           # "inline" | "reference"

    # Images
    keep_inline_images_in: list[str] = field(default_factory=list)
    skip_images: bool = False
    extract_images: bool = False
    max_image_size: int = 5_242_880     # 5 MiB
    capture_svg: bool = False
    infer_dimensions: bool = True

    # Tables
    br_in_tables: bool = False

    # Highlight
    highlight_style: str = "double_equal"  # "double_equal" | "html" | "bold" | "none"

    # Metadata
    extract_metadata: bool = True

    # Whitespace
    whitespace_mode: str = "normalized"  # "normalized" | "strict"
    strip_newlines: bool = False

    # Wrapping
    wrap: bool = False
    wrap_width: int = 80

    # Element handling
    strip_tags: list[str] = field(default_factory=list)
    preserve_tags: list[str] = field(default_factory=list)
    convert_as_inline: bool = False

    # Subscript / superscript
    sub_symbol: str = ""
    sup_symbol: str = ""

    # Newlines
    newline_style: str = "spaces"        # "spaces" | "backslash"

    # Output format
    output_format: str = "markdown"      # "markdown" | "djot" | "plain"

    # Document structure
    include_document_structure: bool = False

    # Preprocessing (pass PreprocessingOptions instance)
    preprocessing: Any | None = None

    # Encoding and debug
    encoding: str = "utf-8"
    debug: bool = False
```

## PreprocessingOptions (dataclass)

```python
from dataclasses import dataclass
from html_to_markdown import PreprocessingOptions

@dataclass
class PreprocessingOptions:
    enabled: bool = True
    preset: str = "standard"             # "minimal" | "standard" | "aggressive"
    remove_navigation: bool = True
    remove_forms: bool = True
```

## Public Types

The following types are exported from `html_to_markdown` directly:

```python
from html_to_markdown import (
    CodeBlockStyle,        # Enum: INDENTED, BACKTICKS, TILDES
    ConversionResult,      # Dataclass: result of convert()
    DocumentMetadata,      # Dataclass: document-level metadata
    HeadingStyle,          # Enum: UNDERLINED, ATX, ATX_CLOSED
    HighlightStyle,        # Enum: DOUBLE_EQUAL, HTML, BOLD, NONE
    HtmlMetadata,          # Dataclass: full metadata extraction result
    LinkStyle,             # Enum: INLINE, REFERENCE
    ListIndentType,        # Enum: SPACES, TABS
    MetadataEntry,         # Dataclass: a single metadata_block document-structure entry
    NewlineStyle,          # Enum: SPACES, BACKSLASH
    OutputFormat,          # Enum: MARKDOWN, DJOT, PLAIN
    PreprocessingPreset,   # Enum: MINIMAL, STANDARD, AGGRESSIVE
    TableGrid,             # Dataclass: table grid structure
    TextDirection,         # Enum: LEFT_TO_RIGHT, RIGHT_TO_LEFT, AUTO
    WhitespaceMode,        # Enum: NORMALIZED, STRICT
)
```

## Accessing Metadata, Tables, and Images

All structured data is in the `ConversionResult` dataclass returned by `convert()`. Use `ConversionOptions` fields to control what is extracted:

```python
from html_to_markdown import convert, ConversionOptions

html = "<h1>Title</h1><table><tr><th>A</th></tr><tr><td>1</td></tr></table>"

# Metadata — enabled by default
result = convert(html)
meta = result.metadata
print(meta.document.title)
print(meta.headers)
print(meta.links)

# Tables and document structure — both require include_document_structure=True
result = convert(html, ConversionOptions(include_document_structure=True))
for table in result.tables:
    print(table.markdown)
doc = result.document

# Plain string output
markdown: str = result.content
```

## Metadata extraction toggle

There is no per-field metadata config — `ConversionOptions.extract_metadata` is a
single on/off switch that populates all of `result.metadata` at once (`document`,
`headers`, `links`, `images`, `structured_data`).

```python
from html_to_markdown import convert, ConversionOptions

html = "<h1>Title</h1>"
result = convert(html, ConversionOptions(extract_metadata=False))
print(result.metadata.headers)  # []

result = convert(html, ConversionOptions(extract_metadata=True))
print([h.text for h in result.metadata.headers])  # ["Title"]
```

## Error Handling

```python
from html_to_markdown import convert
from html_to_markdown.exceptions import (
    ConversionError,       # Base exception for all conversion errors
    ParseError,            # HTML parsing error
    SanitizationError,     # HTML sanitization error
    ConfigError,           # Invalid configuration
    IoError,               # I/O error
    PanicError,            # Panic caught during conversion
    InvalidInputError,     # Invalid input data (binary data, invalid UTF-8)
    OtherError,            # Generic conversion error
)

html = "<h1>Title</h1>"
try:
    result = convert(html)
except InvalidInputError as e:
    print(f"Invalid input: {e}")
except ConfigError as e:
    print(f"Bad options: {e}")
except ConversionError as e:
    print(f"Conversion failed: {e}")
```

## Async Tip

The `convert()` function is synchronous but releases the GIL (Python GVL) during the Rust computation. For CPU-bound workloads, use `asyncio.to_thread()` or a thread pool:

```python
import asyncio
from html_to_markdown import convert

async def convert_async(html: str):
    return await asyncio.to_thread(convert, html)
```
