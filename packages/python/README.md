# html-to-markdown

<div align="center" style="display: flex; flex-wrap: wrap; gap: 8px; justify-content: center; margin: 20px 0;">
  <!-- Language Bindings -->
  <a href="https://crates.io/crates/html-to-markdown-rs">
    <img src="https://img.shields.io/crates/v/html-to-markdown-rs?label=Rust&color=007ec6" alt="Rust">
  </a>
  <a href="https://pypi.org/project/html-to-markdown/">
    <img src="https://img.shields.io/pypi/v/html-to-markdown?label=Python&color=007ec6" alt="Python">
  </a>
  <a href="https://www.npmjs.com/package/@kreuzberg/html-to-markdown-node">
    <img src="https://img.shields.io/npm/v/@kreuzberg/html-to-markdown-node?label=Node.js&color=007ec6" alt="Node.js">
  </a>
  <a href="https://www.npmjs.com/package/@kreuzberg/html-to-markdown-wasm">
    <img src="https://img.shields.io/npm/v/@kreuzberg/html-to-markdown-wasm?label=WASM&color=007ec6" alt="WASM">
  </a>
  <a href="https://central.sonatype.com/artifact/dev.kreuzberg/html-to-markdown">
    <img src="https://img.shields.io/maven-central/v/dev.kreuzberg/html-to-markdown?label=Java&color=007ec6" alt="Java">
  </a>
  <a href="https://pkg.go.dev/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown">
    <img src="https://img.shields.io/badge/Go-v2.19.0-007ec6" alt="Go">
  </a>
  <a href="https://www.nuget.org/packages/KreuzbergDev.HtmlToMarkdown/">
    <img src="https://img.shields.io/nuget/v/KreuzbergDev.HtmlToMarkdown?label=C%23&color=007ec6" alt="C#">
  </a>
  <a href="https://packagist.org/packages/goldziher/html-to-markdown">
    <img src="https://img.shields.io/packagist/v/goldziher/html-to-markdown?label=PHP&color=007ec6" alt="PHP">
  </a>
  <a href="https://rubygems.org/gems/html-to-markdown">
    <img src="https://img.shields.io/gem/v/html-to-markdown?label=Ruby&color=007ec6" alt="Ruby">
  </a>
  <a href="https://hex.pm/packages/html_to_markdown">
    <img src="https://img.shields.io/hexpm/v/html_to_markdown?label=Elixir&color=007ec6" alt="Elixir">
  </a>

  <!-- Project Info -->
  <a href="https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  </a>
</div>

<img width="1128" height="191" alt="html-to-markdown" src="https://github.com/user-attachments/assets/419fc06c-8313-4324-b159-4b4d3cfce5c0" />

<div align="center" style="margin-top: 20px;">
  <a href="https://discord.gg/pXxagNK2zN">
      <img height="22" src="https://img.shields.io/badge/Discord-Join%20our%20community-7289da?logo=discord&logoColor=white" alt="Discord">
  </a>
</div>


High-performance HTML to Markdown converter with a clean Python API (powered by a Rust core).
The same engine also drives the Node.js, Ruby, PHP, and WebAssembly bindings, so rendered Markdown
stays identical across runtimes. Wheels are published for Linux, macOS, and Windows.


## Installation

```bash
pip install html-to-markdown
```


Requires Python 3.10+. Wheels are published for Linux, macOS, and Windows on PyPI.






## Performance Snapshot

Apple M4 • Real Wikipedia documents • `convert()` (Python)

| Document | Size | Latency | Throughput |
| -------- | ---- | ------- | ---------- |
| Lists (Timeline) | 129KB | 0.62ms | 208 MB/s |
| Tables (Countries) | 360KB | 2.02ms | 178 MB/s |
| Mixed (Python wiki) | 656KB | 4.56ms | 144 MB/s |


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


**`convert(html: str, options?: ConversionOptions) -> str`**

Basic HTML-to-Markdown conversion. Fast and simple.

**`convert_with_metadata(html: str, options?: ConversionOptions, metadata_config?: MetadataConfig) -> tuple[str, dict]`**

Extract Markdown plus metadata (headers, links, images, structured data) in a single pass. See [Metadata Extraction Guide](../../examples/metadata-extraction/).

**`convert_with_visitor(html: str, visitor: object, options?: ConversionOptions) -> str`**

Customize conversion with visitor callbacks for element interception. See [Visitor Pattern Guide](../../examples/visitor-pattern/).

**`convert_with_async_visitor(html: str, visitor: object, options?: ConversionOptions) -> str`**

Async version of visitor pattern with asyncio integration.

**`convert_with_inline_images(html: str, image_config?: InlineImageConfig) -> tuple[str, list[dict], list[str]]`**

Extract base64-encoded inline images with metadata.



### Options

**`ConversionOptions`** – Key configuration fields:
- `heading_style`: Heading format (`"underlined"` | `"atx"` | `"atx_closed"`) — default: `"underlined"`
- `list_indent_width`: Spaces per indent level — default: `2`
- `bullets`: Bullet characters cycle — default: `"*+-"`
- `wrap`: Enable text wrapping — default: `false`
- `wrap_width`: Wrap at column — default: `80`
- `code_language`: Default fenced code block language — default: none
- `extract_metadata`: Embed metadata as YAML frontmatter — default: `false`

**`MetadataConfig`** – Selective metadata extraction:
- `extract_headers`: h1-h6 elements — default: `true`
- `extract_links`: Hyperlinks — default: `true`
- `extract_images`: Image elements — default: `true`
- `extract_structured_data`: JSON-LD, Microdata, RDFa — default: `true`
- `max_structured_data_size`: Size limit in bytes — default: `100KB`



## Metadata Extraction

The metadata extraction feature enables comprehensive document analysis during conversion. Extract document properties, headers, links, images, and structured data in a single pass.

**Use Cases:**
- **SEO analysis** – Extract title, description, Open Graph tags, Twitter cards
- **Table of contents generation** – Build structured outlines from heading hierarchy
- **Content migration** – Document all external links and resources
- **Accessibility audits** – Check for images without alt text, empty links, invalid heading hierarchy
- **Link validation** – Classify and validate anchor, internal, external, email, and phone links

**Zero Overhead When Disabled:** Metadata extraction adds negligible overhead and happens during the HTML parsing pass. Disable unused metadata types in `MetadataConfig` to optimize further.

### Example: Quick Start


```python
from html_to_markdown import convert_with_metadata

html = '<h1>Article</h1><img src="test.jpg" alt="test">'
markdown, metadata = convert_with_metadata(html)

print(metadata["document"]["title"])      # Document title
print(metadata["headers"])                 # All h1-h6 elements
print(metadata["links"])                   # All hyperlinks
print(metadata["images"])                  # All images with alt text
print(metadata["structured_data"])        # JSON-LD, Microdata, RDFa
```



For detailed examples including SEO extraction, table-of-contents generation, link validation, and accessibility audits, see the [Metadata Extraction Guide](../../examples/metadata-extraction/).




## Visitor Pattern

The visitor pattern enables custom HTML→Markdown conversion logic by providing callbacks for specific HTML elements during traversal. Use visitors to transform content, filter elements, validate structure, or collect analytics.

**Use Cases:**
- **Custom Markdown dialects** – Convert to Obsidian, Notion, or other flavors
- **Content filtering** – Remove tracking pixels, ads, or unwanted elements
- **URL rewriting** – Rewrite CDN URLs, add query parameters, validate links
- **Accessibility validation** – Check alt text, heading hierarchy, link text
- **Analytics** – Track element usage, link destinations, image sources

**Supported Visitor Methods:** 40+ callbacks for text, inline elements, links, images, headings, lists, blocks, and tables.

### Example: Quick Start


```python
from html_to_markdown import convert_with_visitor

class MyVisitor:
    def visit_link(self, ctx, href, text, title):
        # Rewrite CDN URLs
        if href.startswith("https://old-cdn.com"):
            href = href.replace("https://old-cdn.com", "https://new-cdn.com")
        return {"type": "custom", "output": f"[{text}]({href})"}

    def visit_image(self, ctx, src, alt, title):
        # Skip tracking pixels
        if "tracking" in src:
            return {"type": "skip"}
        return {"type": "continue"}

html = '<a href="https://old-cdn.com/file.pdf">Download</a>'
markdown = convert_with_visitor(html, visitor=MyVisitor())
```

Async support:
```python
import asyncio
from html_to_markdown import convert_with_async_visitor

class AsyncVisitor:
    async def visit_link(self, ctx, href, text, title):
        is_valid = await validate_url(href)
        if not is_valid:
            return {"type": "error", "message": f"Broken link: {href}"}
        return {"type": "continue"}

markdown = await convert_with_async_visitor(html, visitor=AsyncVisitor())
```



For comprehensive examples including content filtering, link footnotes, accessibility validation, and asynchronous URL validation, see the [Visitor Pattern Guide](../../examples/visitor-pattern/).



## Examples

- [Visitor Pattern Guide](../../examples/visitor-pattern/)
- [Metadata Extraction Guide](../../examples/metadata-extraction/)
- [Performance Guide](../../examples/performance/)

## Links

- **GitHub:** [github.com/kreuzberg-dev/html-to-markdown](https://github.com/kreuzberg-dev/html-to-markdown)

- **PyPI:** [pypi.org/project/html-to-markdown](https://pypi.org/project/html-to-markdown/)

- **Kreuzberg Ecosystem:** [kreuzberg.dev](https://kreuzberg.dev)
- **Discord:** [discord.gg/pXxagNK2zN](https://discord.gg/pXxagNK2zN)

## Contributing

We welcome contributions! Please see our [Contributing Guide](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/CONTRIBUTING.md) for details on:

- Setting up the development environment
- Running tests locally
- Submitting pull requests
- Reporting issues

All contributions must follow our code quality standards (enforced via pre-commit hooks):

- Proper test coverage (Rust 95%+, language bindings 80%+)
- Formatting and linting checks
- Documentation for public APIs

## License

MIT License – see [LICENSE](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE).

## Support

If you find this library useful, consider [sponsoring the project](https://github.com/sponsors/kreuzberg-dev).

Have questions or run into issues? We're here to help:

- **GitHub Issues:** [github.com/kreuzberg-dev/html-to-markdown/issues](https://github.com/kreuzberg-dev/html-to-markdown/issues)
- **Discussions:** [github.com/kreuzberg-dev/html-to-markdown/discussions](https://github.com/kreuzberg-dev/html-to-markdown/discussions)
- **Discord Community:** [discord.gg/pXxagNK2zN](https://discord.gg/pXxagNK2zN)
