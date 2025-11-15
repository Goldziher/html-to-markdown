# html-to-markdown

High-performance HTML to Markdown converter with a clean Python API (powered by a Rust core). The same engine also drives the Node.js, Ruby, PHP, and WebAssembly bindings, so rendered Markdown stays identical across runtimes. Wheels are published for Linux, macOS, and Windows.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown.svg)](https://crates.io/crates/html-to-markdown)
[![npm (node)](https://badge.fury.io/js/html-to-markdown-node.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://badge.fury.io/js/html-to-markdown-wasm.svg)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/HtmlToMarkdown.svg)](https://www.nuget.org/packages/HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

## Installation

```bash
pip install html-to-markdown
```

## Performance Snapshot

Apple M4 • Real Wikipedia documents • `convert()` (Python)

| Document            | Size  | Latency | Throughput | Docs/sec |
| ------------------- | ----- | ------- | ---------- | -------- |
| Lists (Timeline)    | 129KB | 0.62ms  | 208 MB/s   | 1,613    |
| Tables (Countries)  | 360KB | 2.02ms  | 178 MB/s   | 495      |
| Mixed (Python wiki) | 656KB | 4.56ms  | 144 MB/s   | 219      |

> V1 averaged ~2.5 MB/s (Python/BeautifulSoup). V2's Rust engine delivers 60–80× higher throughput.

### Benchmark Fixtures (Apple M4)

Pulled directly from `tools/runtime-bench` (`task bench:bindings -- --language python`) so they stay in lockstep with the Rust core:

| Document               | Size   | ops/sec (Python) |
| ---------------------- | ------ | ---------------- |
| Lists (Timeline)       | 129 KB | 1,405            |
| Tables (Countries)     | 360 KB | 352              |
| Medium (Python)        | 657 KB | 158              |
| Large (Rust)           | 567 KB | 183              |
| Small (Intro)          | 463 KB | 223              |
| hOCR German PDF        | 44 KB  | 2,991            |
| hOCR Invoice           | 4 KB   | 23,500           |
| hOCR Embedded Tables   | 37 KB  | 3,464            |

> Re-run locally with `task bench:bindings -- --language python --output tmp.json` to compare against CI history.

## Quick Start

```python
from html_to_markdown import convert

html = """
<h1>Welcome</h1>
<p>This is <strong>fast</strong> Rust-powered conversion!</p>
<ul>
    <li>Blazing fast</li>
    <li>Type safe</li>
    <li>Easy to use</li>
</ul>
"""

markdown = convert(html)
print(markdown)
```

## Configuration (v2 API)

```python
from html_to_markdown import ConversionOptions, convert

options = ConversionOptions(
    heading_style="atx",
    list_indent_width=2,
    bullets="*+-",
)
options.escape_asterisks = True
options.code_language = "python"
options.extract_metadata = True

markdown = convert(html, options)
```

### Reusing Parsed Options

Avoid re-parsing the same option dictionaries inside hot loops by building a reusable handle:

```python
from html_to_markdown import ConversionOptions, convert_with_handle, create_options_handle

handle = create_options_handle(ConversionOptions(hocr_spatial_tables=False))

for html in documents:
    markdown = convert_with_handle(html, handle)
```

### HTML Preprocessing

```python
from html_to_markdown import ConversionOptions, PreprocessingOptions, convert

options = ConversionOptions(
    preprocessing=PreprocessingOptions(enabled=True, preset="aggressive"),
)

markdown = convert(scraped_html, options)
```

### Inline Image Extraction

```python
from html_to_markdown import InlineImageConfig, convert_with_inline_images

markdown, inline_images, warnings = convert_with_inline_images(
    '<p><img src="data:image/png;base64,...==" alt="Pixel" width="1" height="1"></p>',
    image_config=InlineImageConfig(max_decoded_size_bytes=1024, infer_dimensions=True),
)

if inline_images:
    first = inline_images[0]
    print(first["format"], first["dimensions"], first["attributes"])  # e.g. "png", (1, 1), {"width": "1"}
```

Each inline image is returned as a typed dictionary (`bytes` payload, metadata, and relevant HTML attributes). Warnings are human-readable skip reasons.

### hOCR (HTML OCR) Support

```python
from html_to_markdown import ConversionOptions, convert

# Default: emit structured Markdown directly
markdown = convert(hocr_html)

# hOCR documents are detected automatically; tables are reconstructed without extra configuration.
markdown = convert(hocr_html)
```

## CLI (same engine)

```bash
pipx install html-to-markdown  # or: pip install html-to-markdown

html-to-markdown page.html > page.md
cat page.html | html-to-markdown --heading-style atx > page.md
```

## API Surface

### `ConversionOptions`

Key fields (see docstring for full matrix):

- `heading_style`: `"underlined" | "atx" | "atx_closed"`
- `list_indent_width`: spaces per indent level (default 2)
- `bullets`: cycle of bullet characters (`"*+-"`)
- `strong_em_symbol`: `"*"` or `"_"`
- `code_language`: default fenced code block language
- `wrap`, `wrap_width`: wrap Markdown output
- `strip_tags`: remove specific HTML tags
- `preprocessing`: `PreprocessingOptions`
- `encoding`: input character encoding (informational)

### `PreprocessingOptions`

- `enabled`: enable HTML sanitisation (default: `True` since v2.4.2 for robust malformed HTML handling)
- `preset`: `"minimal" | "standard" | "aggressive"` (default: `"standard"`)
- `remove_navigation`: remove navigation elements (default: `True`)
- `remove_forms`: remove form elements (default: `True`)

**Note:** As of v2.4.2, preprocessing is enabled by default to ensure robust handling of malformed HTML (e.g., bare angle brackets like `1<2` in content). Set `enabled=False` if you need minimal preprocessing.

### `InlineImageConfig`

- `max_decoded_size_bytes`: reject larger payloads
- `filename_prefix`: generated name prefix (`embedded_image` default)
- `capture_svg`: collect inline `<svg>` (default `True`)
- `infer_dimensions`: decode raster images to obtain dimensions (default `False`)

## Performance: V2 vs V1 Compatibility Layer

### ⚠️ Important: Always Use V2 API

The v2 API (`convert()`) is **strongly recommended** for all code. The v1 compatibility layer adds significant overhead and should only be used for gradual migration:

```python
# ✅ RECOMMENDED - V2 Direct API (Fast)
from html_to_markdown import convert, ConversionOptions

markdown = convert(html)  # Simple conversion - FAST
markdown = convert(html, ConversionOptions(heading_style="atx"))  # With options - FAST

# ❌ AVOID - V1 Compatibility Layer (Slow)
from html_to_markdown import convert_to_markdown

markdown = convert_to_markdown(html, heading_style="atx")  # Adds 77% overhead
```

### Performance Comparison

Benchmarked on Apple M4 with 25-paragraph HTML document:

| API                      | ops/sec          | Relative Performance | Recommendation      |
| ------------------------ | ---------------- | -------------------- | ------------------- |
| **V2 API** (`convert()`) | **129,822**      | baseline             | ✅ **Use this**     |
| **V1 Compat Layer**      | **67,673**       | **77% slower**       | ⚠️ Migration only   |
| **CLI**                  | **150-210 MB/s** | Fastest              | ✅ Batch processing |

The v1 compatibility layer creates extra Python objects and performs additional conversions, significantly impacting performance.

### When to Use Each

- **V2 API (`convert()`)**: All new code, production systems, performance-critical applications ← **Use this**
- **V1 Compat (`convert_to_markdown()`)**: Only for gradual migration from legacy codebases
- **CLI (`html-to-markdown`)**: Batch processing, shell scripts, maximum throughput

## v1 Compatibility

A compatibility layer is provided to ease migration from v1.x:

- **Compat shim**: `html_to_markdown.v1_compat` exposes `convert_to_markdown`, `convert_to_markdown_stream`, and `markdownify`. Keyword mappings are listed in the [changelog](https://github.com/Goldziher/html-to-markdown/blob/main/CHANGELOG.md#v200).
- **⚠️ Performance warning**: These compatibility functions add 77% overhead. Migrate to v2 API as soon as possible.
- **CLI**: The Rust CLI replaces the old Python script. New flags are documented via `html-to-markdown --help`.
- **Removed options**: `code_language_callback`, `strip`, and streaming APIs were removed; use `ConversionOptions`, `PreprocessingOptions`, and the inline-image helpers instead.

## Links

- GitHub: [https://github.com/Goldziher/html-to-markdown](https://github.com/Goldziher/html-to-markdown)
- Discord: [https://discord.gg/pXxagNK2zN](https://discord.gg/pXxagNK2zN)
- Kreuzberg ecosystem: [https://kreuzberg.dev](https://kreuzberg.dev)

## License

MIT License – see [LICENSE](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE).

## Support

If you find this library useful, consider [sponsoring the project](https://github.com/sponsors/Goldziher).
