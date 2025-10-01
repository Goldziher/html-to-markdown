# html-to-markdown v2 API Design

## Overview

V2 replaces BeautifulSoup with **lol-html** (Rust streaming HTML parser) for better performance and memory efficiency. This document outlines the API changes and architecture.

## Current API (v1)

### Main Functions

- `convert_to_markdown(source, **options)` → `str`
- `convert_to_markdown_stream(source, **options)` → `Generator[str, None, None]`

### Issues with Current API

1. **Too many parameters** (~40 keyword arguments)
1. **BeautifulSoup dependency** (DOM-based, memory-intensive)
1. **Inconsistent streaming** (streaming is bolted on, not native)
1. **No clear separation** between parsing options and conversion options

## New API (v2)

### Core Principles

1. **Streaming-first**: Built on lol-html's streaming parser
1. **Options as dataclasses**: Group related options into typed configuration objects
1. **Functional API**: Pure functions, no classes
1. **Simpler API surface**: Fewer top-level functions, clearer intent
1. **Parser-agnostic**: No BeautifulSoup in public API

### Main API

```python
from html_to_markdown import convert, convert_stream, ConversionOptions

# Simple usage
markdown = convert(html)

# With options
options = ConversionOptions(
    heading_style="atx",
    list_indent_width=2,
    extract_metadata=True,
)
markdown = convert(html, options)

# Streaming
for chunk in convert_stream(html, options):
    print(chunk, end="")
```

### Configuration Objects

```python
@dataclass
class ConversionOptions:
    """Main conversion configuration."""

    # Heading options
    heading_style: Literal["atx", "atx_closed", "underlined"] = "atx"

    # List options
    list_indent_type: Literal["spaces", "tabs"] = "spaces"
    list_indent_width: int = 4
    bullets: str = "*+-"

    # Text formatting
    strong_em_symbol: Literal["*", "_"] = "*"
    escape_asterisks: bool = True
    escape_underscores: bool = True
    escape_misc: bool = True

    # Code blocks
    code_language: str = ""
    code_language_callback: Callable[[str], str] | None = None

    # Links
    autolinks: bool = True

    # Images
    keep_inline_images_in: set[str] | None = None

    # Tables
    br_in_tables: bool = False

    # Highlighting
    highlight_style: Literal["double-equal", "html", "bold"] = "double-equal"

    # Metadata
    extract_metadata: bool = True
    default_title: bool = False

    # Whitespace
    whitespace_mode: Literal["normalized", "strict"] = "normalized"
    strip_newlines: bool = False

    # Wrapping
    wrap: bool = False
    wrap_width: int = 80

    # Element filtering
    convert: set[str] | None = None
    strip: set[str] | None = None
    convert_as_inline: bool = False

    # Subscript/superscript
    sub_symbol: str = ""
    sup_symbol: str = ""

    # Newlines
    newline_style: Literal["spaces", "backslash"] = "spaces"

@dataclass
class PreprocessingOptions:
    """HTML preprocessing configuration."""

    enabled: bool = False
    preset: Literal["minimal", "standard", "aggressive"] = "standard"
    remove_navigation: bool = True
    remove_forms: bool = True
    excluded_navigation_classes: set[str] | None = None
    extra_navigation_classes: set[str] | None = None

@dataclass
class ParsingOptions:
    """HTML parsing configuration."""

    encoding: str = "utf-8"
    detect_encoding: bool = True

@dataclass
class StreamingOptions:
    """Streaming conversion configuration."""

    chunk_size: int = 8192
    progress_callback: Callable[[int, int], None] | None = None
```

### Core Functions

```python
def convert(
    html: str | bytes,
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
    parsing: ParsingOptions | None = None,
    custom_handlers: dict[str, ElementHandlerFn] | None = None,
) -> str:
    """Convert HTML to Markdown.

    Args:
        html: HTML string or bytes
        options: Conversion options
        preprocessing: Preprocessing options
        parsing: Parsing options
        custom_handlers: Custom element handler functions

    Returns:
        Markdown string

    Raises:
        EmptyHtmlError: If HTML is empty
        ConversionError: If conversion fails

    Example:
        >>> options = ConversionOptions(heading_style="atx")
        >>> convert("<h1>Title</h1>", options)
        '# Title\\n\\n'
    """

def convert_stream(
    html: str | bytes,
    options: ConversionOptions | None = None,
    preprocessing: PreprocessingOptions | None = None,
    parsing: ParsingOptions | None = None,
    streaming: StreamingOptions | None = None,
    custom_handlers: dict[str, ElementHandlerFn] | None = None,
) -> Generator[str, None, None]:
    """Convert HTML to Markdown with streaming.

    Args:
        html: HTML string or bytes
        options: Conversion options
        preprocessing: Preprocessing options
        parsing: Parsing options
        streaming: Streaming options
        custom_handlers: Custom element handler functions

    Yields:
        Markdown chunks

    Example:
        >>> for chunk in convert_stream(html):
        ...     print(chunk, end='')
    """
```

### Custom Element Handlers

Element handlers are pure functions that take context and return markdown:

```python
from html_to_markdown import ElementContext, ElementHandlerFn

# Type alias for handler functions
ElementHandlerFn = Callable[[ElementContext], str]

def custom_link_handler(ctx: ElementContext) -> str:
    """Custom handler for <a> tags.

    Args:
        ctx: Element context with tag name, attributes, text content

    Returns:
        Markdown string for this element
    """
    href = ctx.get_attr("href")
    text = ctx.text
    title = ctx.get_attr("title")

    if title:
        return f'[{text}]({href} "{title}")'
    return f"[{text}]({href})"

# Use custom handler
markdown = convert(html, custom_handlers={"a": custom_link_handler})
```

Handler functions can also be closures for configuration:

```python
def make_link_handler(add_icon: bool = False) -> ElementHandlerFn:
    """Factory for creating configured link handlers."""

    def handler(ctx: ElementContext) -> str:
        href = ctx.get_attr("href")
        text = ctx.text
        icon = "🔗 " if add_icon else ""
        return f"{icon}[{text}]({href})"
    return handler

# Use configured handler
markdown = convert(html, custom_handlers={"a": make_link_handler(add_icon=True)})
```

### Backward Compatibility

For users migrating from v1, we keep the old function name:

```python
def convert_to_markdown(source: str | bytes, **kwargs) -> str:
    """Convert HTML to Markdown (v1 compatibility).

    This function accepts the v1 keyword arguments and maps them
    to the new options-based API.

    DEPRECATED: Use convert() with ConversionOptions instead.
    """
    warnings.warn(
        "convert_to_markdown() with kwargs is deprecated, " "use convert() with ConversionOptions instead",
        DeprecationWarning,
        stacklevel=2,
    )

    # Map v1 kwargs to v2 options
    options = _map_legacy_options(**kwargs)
    return convert(source, options)
```

## Architecture Changes

### v1 Architecture (BeautifulSoup)

```
HTML → BeautifulSoup → DOM Tree → Walk Tree → Convert Each Tag → Markdown
```

### v2 Architecture (lol-html)

```
HTML → lol-html Streaming Parser → Element Handlers → Markdown Chunks → Markdown
                                    ↑
                                    |
                            Event-driven processing
```

### Benefits

1. **Memory efficiency**: No full DOM tree in memory
1. **Streaming native**: True streaming support
1. **Performance**: lol-html is 2x faster than html5ever
1. **Type safety**: Dataclasses provide better IDE support
1. **Extensibility**: Element handlers are cleaner than converter functions

## Migration Path

### Stage 1: Python API Refactor + Baseline Benchmarks

1. Update to functional API with dataclass options
1. Ensure all existing tests pass
1. Add comprehensive pytest-benchmark suite
1. Download Wikipedia test documents
1. Run benchmarks and save baseline results
1. Commit with benchmark baseline (tag: v2.0.0-alpha.1)

### Stage 2: Rust Backend Implementation

1. Implement lol-html + ammonia backend in Rust
1. Replace nh3 (Python bindings) with direct ammonia
1. Create PyO3 bindings
1. Port all features (hOCR, streaming, metadata, etc.)
1. Run benchmarks and compare to baseline
1. Commit with performance comparison (tag: v2.0.0-beta.1)

### Stage 3: Documentation & Release

1. Update documentation and migration guide
1. Update CHANGELOG.md with nh3→ammonia migration notes
1. Release v2.0.0

See `IMPLEMENTATION_PLAN.md` for detailed breakdown.

## Open Questions

1. **How to handle hOCR documents in streaming mode?**
   - Option A: Detect hOCR in first pass, switch to DOM mode if needed
   - Option B: Implement hOCR detection/processing in streaming (more complex)
   - **Recommendation**: Option A - hOCR is rare, DOM mode acceptable

1. **Should we expose lol-html element directly to handlers?**
   - Option A: Wrap in ElementContext abstraction (easier to maintain/test)
   - Option B: Direct access to lol-html Element (more powerful, less portable)
   - **Recommendation**: Option A - abstraction allows backend changes

1. **Preprocessing in streaming mode?**
   - Option A: Run preprocessing on full HTML first (requires buffering)
   - Option B: Disable preprocessing for streaming (document clearly)
   - **Recommendation**: Option A for convert(), Option B for convert_stream()
