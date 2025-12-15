# html-to-markdown

High-performance HTML to Markdown converter with a clean Python API (powered by a Rust core). The same engine also drives the Node.js, Ruby, PHP, and WebAssembly bindings, so rendered Markdown stays identical across runtimes. Wheels are published for Linux, macOS, and Windows.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg?logo=rust&label=crates.io)](https://crates.io/crates/html-to-markdown-rs)
[![npm (node)](https://img.shields.io/npm/v/html-to-markdown-node.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://img.shields.io/npm/v/html-to-markdown-wasm.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://img.shields.io/pypi/v/html-to-markdown.svg?logo=pypi)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/Goldziher.HtmlToMarkdown.svg)](https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown)
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
    ...
)

preprocessing = PreprocessingOptions(
    enabled=True,
    preset="aggressive",
)

markdown = convert(scraped_html, options, preprocessing)
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

### Metadata Extraction

Extract comprehensive metadata (title, description, headers, links, images, structured data) during conversion in a single pass.

#### Basic Usage

```python
from html_to_markdown import convert_with_metadata

html = """
<html>
  <head>
    <title>Example Article</title>
    <meta name="description" content="Demo page">
    <link rel="canonical" href="https://example.com/article">
  </head>
  <body>
    <h1 id="welcome">Welcome</h1>
    <a href="https://example.com" rel="nofollow external">Example link</a>
    <img src="https://example.com/image.jpg" alt="Hero" width="640" height="480">
  </body>
</html>
"""

markdown, metadata = convert_with_metadata(html)

print(markdown)
print(metadata["document"]["title"])       # "Example Article"
print(metadata["headers"][0]["text"])      # "Welcome"
print(metadata["links"][0]["href"])        # "https://example.com"
print(metadata["images"][0]["dimensions"]) # (640, 480)
```

#### Configuration

Control which metadata types are extracted using `MetadataConfig`:

```python
from html_to_markdown import ConversionOptions, MetadataConfig, convert_with_metadata

options = ConversionOptions(heading_style="atx")
config = MetadataConfig(
    extract_headers=True,           # h1-h6 elements (default: True)
    extract_links=True,             # <a> hyperlinks (default: True)
    extract_images=True,            # <img> elements (default: True)
    extract_structured_data=True,   # JSON-LD, Microdata, RDFa (default: True)
    max_structured_data_size=1_000_000,  # Max bytes for structured data (default: 100KB)
)

markdown, metadata = convert_with_metadata(html, options, config)
```

#### Metadata Structure

The `metadata` dictionary contains five categories:

```python
metadata = {
    "document": {                    # Document-level metadata from <head>
        "title": str | None,
        "description": str | None,
        "keywords": list[str],       # Comma-separated keywords from meta tags
        "author": str | None,
        "canonical_url": str | None, # link[rel="canonical"] href
        "base_href": str | None,
        "language": str | None,      # lang attribute (e.g., "en")
        "text_direction": str | None, # "ltr", "rtl", or "auto"
        "open_graph": dict[str, str], # og:* meta properties
        "twitter_card": dict[str, str], # twitter:* meta properties
        "meta_tags": dict[str, str],  # Other meta tag properties
    },
    "headers": [                     # h1-h6 elements with hierarchy
        {
            "level": int,            # 1-6
            "text": str,             # Normalized text content
            "id": str | None,        # HTML id attribute
            "depth": int,            # Nesting depth in document tree
            "html_offset": int,      # Byte offset in original HTML
        },
        # ... more headers
    ],
    "links": [                       # Extracted <a> elements
        {
            "href": str,
            "text": str,
            "title": str | None,
            "link_type": str,        # "anchor" | "internal" | "external" | "email" | "phone" | "other"
            "rel": list[str],        # rel attribute values
            "attributes": dict[str, str],  # Other HTML attributes
        },
        # ... more links
    ],
    "images": [                      # Extracted <img> elements
        {
            "src": str,              # Image source (URL or data URI)
            "alt": str | None,
            "title": str | None,
            "dimensions": tuple[int, int] | None,  # (width, height)
            "image_type": str,       # "data_uri" | "inline_svg" | "external" | "relative"
            "attributes": dict[str, str],
        },
        # ... more images
    ],
    "structured_data": [             # JSON-LD, Microdata, RDFa blocks
        {
            "data_type": str,        # "json_ld" | "microdata" | "rdfa"
            "raw_json": str,         # JSON string representation
            "schema_type": str | None,  # Detected schema type (e.g., "Article")
        },
        # ... more structured data
    ],
}
```

#### Real-World Use Cases

**Extract Article Metadata for SEO**

```python
from html_to_markdown import convert_with_metadata

def extract_article_metadata(html: str) -> dict:
    markdown, metadata = convert_with_metadata(html)
    doc = metadata["document"]

    return {
        "title": doc.get("title"),
        "description": doc.get("description"),
        "keywords": doc.get("keywords", []),
        "author": doc.get("author"),
        "canonical_url": doc.get("canonical_url"),
        "language": doc.get("language"),
        "open_graph": doc.get("open_graph", {}),
        "twitter_card": doc.get("twitter_card", {}),
        "markdown": markdown,
    }

# Usage
seo_data = extract_article_metadata(html)
print(f"Title: {seo_data['title']}")
print(f"Language: {seo_data['language']}")
print(f"OG Image: {seo_data['open_graph'].get('image')}")
```

**Build Table of Contents**

```python
from html_to_markdown import convert_with_metadata

def build_table_of_contents(html: str) -> list[dict]:
    """Generate a nested TOC from header structure."""
    markdown, metadata = convert_with_metadata(html)
    headers = metadata["headers"]

    toc = []
    for header in headers:
        toc.append({
            "level": header["level"],
            "text": header["text"],
            "anchor": header.get("id") or header["text"].lower().replace(" ", "-"),
        })
    return toc

# Usage
toc = build_table_of_contents(html)
for item in toc:
    indent = "  " * (item["level"] - 1)
    print(f"{indent}- [{item['text']}](#{item['anchor']})")
```

**Validate Links and Accessibility**

```python
from html_to_markdown import convert_with_metadata

def check_accessibility(html: str) -> dict:
    """Find common accessibility and SEO issues."""
    markdown, metadata = convert_with_metadata(html)

    return {
        "images_without_alt": [
            img for img in metadata["images"]
            if not img.get("alt")
        ],
        "links_without_text": [
            link for link in metadata["links"]
            if not link.get("text", "").strip()
        ],
        "external_links_count": len([
            link for link in metadata["links"]
            if link["link_type"] == "external"
        ]),
        "broken_anchors": [
            link for link in metadata["links"]
            if link["link_type"] == "anchor"
        ],
    }

# Usage
issues = check_accessibility(html)
if issues["images_without_alt"]:
    print(f"Found {len(issues['images_without_alt'])} images without alt text")
```

**Extract Structured Data (JSON-LD, Microdata)**

```python
from html_to_markdown import convert_with_metadata
import json

def extract_json_ld_schemas(html: str) -> list[dict]:
    """Extract all JSON-LD structured data blocks."""
    markdown, metadata = convert_with_metadata(html)

    schemas = []
    for block in metadata["structured_data"]:
        if block["data_type"] == "json_ld":
            try:
                schema = json.loads(block["raw_json"])
                schemas.append({
                    "type": block.get("schema_type"),
                    "data": schema,
                })
            except json.JSONDecodeError:
                continue
    return schemas

# Usage
schemas = extract_json_ld_schemas(html)
for schema in schemas:
    print(f"Found {schema['type']} schema:")
    print(json.dumps(schema["data"], indent=2))
```

**Migrate Content with Preservation of Links and Images**

```python
from html_to_markdown import convert_with_metadata

def migrate_with_manifest(html: str, base_url: str) -> tuple[str, dict]:
    """Convert to Markdown while capturing all external references."""
    markdown, metadata = convert_with_metadata(html)

    manifest = {
        "title": metadata["document"].get("title"),
        "external_links": [
            {"url": link["href"], "text": link["text"]}
            for link in metadata["links"]
            if link["link_type"] == "external"
        ],
        "external_images": [
            {"url": img["src"], "alt": img.get("alt")}
            for img in metadata["images"]
            if img["image_type"] == "external"
        ],
    }
    return markdown, manifest

# Usage
md, manifest = migrate_with_manifest(html, "https://example.com")
print(f"Converted: {manifest['title']}")
print(f"External resources: {len(manifest['external_links'])} links, {len(manifest['external_images'])} images")
```

#### Feature Detection

Check if metadata extraction is available at runtime:

```python
from html_to_markdown import convert_with_metadata, convert

try:
    # Try to use metadata extraction
    markdown, metadata = convert_with_metadata(html)
    print(f"Metadata available: {metadata['document'].get('title')}")
except (NameError, TypeError):
    # Fallback for builds without metadata feature
    markdown = convert(html)
    print("Metadata feature not available, using basic conversion")
```

#### Error Handling

Metadata extraction is designed to be robust:

```python
from html_to_markdown import convert_with_metadata, MetadataConfig

# Handle large structured data safely
config = MetadataConfig(
    extract_structured_data=True,
    max_structured_data_size=500_000,  # 500KB limit
)

try:
    markdown, metadata = convert_with_metadata(html, metadata_config=config)

    # Safe access with defaults
    title = metadata["document"].get("title", "Untitled")
    headers = metadata["headers"] or []
    images = metadata["images"] or []

except Exception as e:
    # Handle parsing errors gracefully
    print(f"Extraction error: {e}")
    # Fallback to basic conversion
    from html_to_markdown import convert
    markdown = convert(html)
```

#### Performance Considerations

1. **Single-Pass Collection**: Metadata extraction happens during HTML parsing with zero overhead when disabled.
2. **Memory Efficient**: Collections use reasonable pre-allocations (32 headers, 64 links, 16 images typical).
3. **Selective Extraction**: Disable unused metadata types in `MetadataConfig` to reduce overhead.
4. **Structured Data Limits**: Large JSON-LD blocks are skipped if they exceed the size limit to prevent memory exhaustion.

```python
from html_to_markdown import MetadataConfig, convert_with_metadata

# Optimize for performance
config = MetadataConfig(
    extract_headers=True,
    extract_links=False,  # Skip if not needed
    extract_images=False, # Skip if not needed
    extract_structured_data=False,  # Skip if not needed
)

markdown, metadata = convert_with_metadata(html, metadata_config=config)
```

#### Differences from Basic Conversion

When `extract_metadata=True` (default in `ConversionOptions`), basic metadata is embedded in a YAML frontmatter block:

```python
from html_to_markdown import convert, ConversionOptions

# Basic metadata as YAML frontmatter
options = ConversionOptions(extract_metadata=True)
markdown = convert(html, options)
# Output: "---\ntitle: ...\n---\n\nContent..."

# Rich metadata extraction (all metadata types)
from html_to_markdown import convert_with_metadata
markdown, full_metadata = convert_with_metadata(html)
# Returns structured data dict with headers, links, images, etc.
```

The two approaches serve different purposes:
- `extract_metadata=True`: Embeds basic metadata in the output Markdown
- `convert_with_metadata()`: Returns structured metadata for programmatic access

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
