# html-to-markdown

High-performance HTML to Markdown converter with a clean Python API (powered by a Rust core). The same engine also drives the Node.js, Ruby, PHP, and WebAssembly bindings, so rendered Markdown stays identical across runtimes. Wheels are published for Linux, macOS, and Windows.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg?logo=rust&label=crates.io)](https://crates.io/crates/html-to-markdown-rs)
[![npm (node)](https://img.shields.io/npm/v/%40kreuzberg%2Fhtml-to-markdown-node.svg?logo=npm)](https://www.npmjs.com/package/@kreuzberg/html-to-markdown-node)
[![npm (wasm)](https://img.shields.io/npm/v/%40kreuzberg%2Fhtml-to-markdown-wasm.svg?logo=npm)](https://www.npmjs.com/package/@kreuzberg/html-to-markdown-wasm)
[![PyPI](https://img.shields.io/pypi/v/html-to-markdown.svg?logo=pypi)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/Goldziher.HtmlToMarkdown.svg)](https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown.svg)](https://pkg.go.dev/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

## Installation

```bash
pip install html-to-markdown
```

Requires Python 3.10+. Wheels are published for Linux, macOS, and Windows on PyPI.

## Performance Snapshot

Apple M4 • Real Wikipedia documents • `convert()` (Python)

| Document            | Size  | Latency | Throughput |
| ------------------- | ----- | ------- | ---------- |
| Lists (Timeline)    | 129KB | 0.62ms  | 208 MB/s   |
| Tables (Countries)  | 360KB | 2.02ms  | 178 MB/s   |
| Mixed (Python wiki) | 656KB | 4.56ms  | 144 MB/s   |

See [Performance Guide](../../examples/performance/) for detailed benchmarks.

## Quick Start

Basic conversion:

```python
from html_to_markdown import convert

html = "<h1>Hello</h1><p>This is <strong>fast</strong>!</p>"
markdown = convert(html)
```

With conversion options:

```python
from html_to_markdown import ConversionOptions, convert

options = ConversionOptions(
    heading_style="atx",
    list_indent_width=2,
)
markdown = convert(html, options)
```

With async support:

```python
import asyncio
from html_to_markdown import convert_with_async_visitor

class AsyncVisitor:
    async def visit_link(self, ctx, href, text, title):
        # Validate URLs asynchronously
        return {"type": "continue"}

markdown = convert_with_async_visitor(html, visitor=AsyncVisitor())
```

## API Reference

### Core Functions

**`convert(html: str, options?: ConversionOptions, preprocessing?: PreprocessingOptions) -> str`**

Basic HTML-to-Markdown conversion. Fast and simple.

**`convert_with_metadata(html: str, options?: ConversionOptions, metadata_config?: MetadataConfig) -> tuple[str, dict]`**

Extract Markdown plus comprehensive metadata (headers, links, images, structured data) in a single pass. See [Metadata Extraction Guide](../../examples/metadata-extraction/) for detailed examples.

**`convert_with_visitor(html: str, visitor: object, options?: ConversionOptions) -> str`**

Customize conversion with visitor callbacks for element interception. Supports custom filtering, validation, and formatting. See [Visitor Pattern Guide](../../examples/visitor-pattern/) for 5+ practical examples and full API.

**`convert_with_async_visitor(html: str, visitor: object, options?: ConversionOptions) -> str`**

Async version of visitor pattern with seamless asyncio integration via pyo3-async-runtimes. Mix sync and async visitor methods freely.

**`convert_with_inline_images(html: str, image_config?: InlineImageConfig) -> tuple[str, list[dict], list[str]]`**

Extract base64-encoded inline images with metadata (format, dimensions, attributes). Returns Markdown, image list, and warnings.

### Options

**`ConversionOptions`** – Key fields:
- `heading_style`: `"underlined" | "atx" | "atx_closed"` (default: `"underlined"`)
- `list_indent_width`: spaces per indent level (default: `2`)
- `bullets`: cycle of bullet characters (default: `"*+-"`)
- `wrap`: enable text wrapping (default: `False`)
- `wrap_width`: wrap at column (default: `80`)
- `code_language`: default fenced code block language
- `extract_metadata`: embed basic metadata as YAML frontmatter (default: `True`)

**`MetadataConfig`** – Selective metadata extraction:
- `extract_headers`: h1-h6 elements (default: `True`)
- `extract_links`: hyperlinks (default: `True`)
- `extract_images`: img elements (default: `True`)
- `extract_structured_data`: JSON-LD, Microdata, RDFa (default: `True`)
- `max_structured_data_size`: size limit in bytes (default: `100KB`)

**`PreprocessingOptions`** – HTML sanitization:
- `enabled`: robust handling of malformed HTML (default: `True` since v2.4.2)
- `preset`: `"minimal" | "standard" | "aggressive"` (default: `"standard"`)

## Examples

Comprehensive guides with Python examples:

- **[Visitor Pattern](../../examples/visitor-pattern/)** – Custom HTML element callbacks, filtering, validation, URL rewriting, accessibility checks, and async I/O integration. 5+ real-world examples included.

- **[Metadata Extraction](../../examples/metadata-extraction/)** – Extract SEO tags, headers, links, images, and structured data. Use cases: SEO analysis, content migration, accessibility audits, table-of-contents generation.

- **[Performance & Benchmarking](../../examples/performance/)** – Benchmarks across languages, optimization tips, and the benchmark harness you can run locally.

## CLI

```bash
pipx install html-to-markdown  # or: pip install html-to-markdown

html-to-markdown page.html > page.md
cat page.html | html-to-markdown --heading-style atx > page.md
html-to-markdown --help
```

## v1 Compatibility

A compatibility layer eases migration from v1.x:

```python
from html_to_markdown.v1_compat import convert_to_markdown

# V1 API (adds ~77% overhead; for gradual migration only)
markdown = convert_to_markdown(html, heading_style="atx")
```

Keyword mappings and removal notes are in the [changelog](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/CHANGELOG.md#v200). Always migrate to the V2 API (`convert()`) for production code.

## Links

- **GitHub:** [github.com/kreuzberg-dev/html-to-markdown](https://github.com/kreuzberg-dev/html-to-markdown)
- **PyPI:** [pypi.org/project/html-to-markdown](https://pypi.org/project/html-to-markdown/)
- **Discord:** [discord.gg/pXxagNK2zN](https://discord.gg/pXxagNK2zN)
- **Kreuzberg Ecosystem:** [kreuzberg.dev](https://kreuzberg.dev)

## License

MIT License – see [LICENSE](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE).

## Support

If you find this library useful, consider [sponsoring the project](https://github.com/sponsors/kreuzberg-dev).
