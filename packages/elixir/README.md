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


Elixir bindings for the Rust html-to-markdown engine. The package exposes a fast HTML to Markdown converter implemented with Rustler.
Ship identical Markdown across every runtime while enjoying native performance with Rustler NIF bindings.


## Installation

```bash
Add {:html_to_markdown, &#34;~&gt; 2.19.0&#34;} to mix.exs deps
```



Requires Elixir 1.19+ and OTP 28. Add to your `mix.exs`:

```elixir
def deps do
  [
    {:html_to_markdown, "~> 2.19.0"}
  ]
end
```






## Performance Snapshot

Apple M4 • Real Wikipedia documents • `convert()` (Elixir)

| Document | Size | Ops/sec | Throughput |
| -------- | ---- | ------- | ---------- |
| Lists (Timeline) | 129KB | 2,547 | 321.7 MB/s |
| Tables (Countries) | 360KB | 835 | 293.8 MB/s |
| Medium (Python) | 656KB | 439 | 281.5 MB/s |
| Large (Rust) | 567KB | 485 | 268.7 MB/s |
| Small (Intro) | 463KB | 581 | 262.9 MB/s |
| HOCR German PDF | 44KB | 7,106 | 303.1 MB/s |
| HOCR Embedded Tables | 37KB | 6,231 | 226.1 MB/s |
| HOCR Invoice | 4KB | 62,657 | 256.4 MB/s |


See [Performance Guide](../../examples/performance/) for detailed benchmarks.


## Quick Start

Basic conversion:

```elixir
iex&gt; {:ok, markdown} = HtmlToMarkdown.convert(&#34;&lt;h1&gt;Hello&lt;/h1&gt;&#34;)
iex&gt; markdown
&#34;# Hello\n&#34;
```



With conversion options:

```elixir
# Pre-build reusable options
iex&gt; handle = HtmlToMarkdown.options(%Options{wrap: true, wrap_width: 40})
iex&gt; HtmlToMarkdown.convert_with_options(&#34;&lt;p&gt;Reusable&lt;/p&gt;&#34;, handle)
{:ok, &#34;Reusable\n&#34;}
```






## API Reference

### Core Functions


**`HtmlToMarkdown.convert(html, options \\ nil) :: String.t()`**

Basic HTML-to-Markdown conversion. Fast and simple.

**`HtmlToMarkdown.convert_with_metadata(html, options \\ nil, config \\ nil) :: {String.t(), map()}`**

Extract Markdown plus metadata in a single pass. See [Metadata Extraction Guide](../../examples/metadata-extraction/).

**`HtmlToMarkdown.convert_with_inline_images(html, config \\ nil) :: {String.t(), list(map()), list(String.t())}`**

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


```elixir
alias HtmlToMarkdown

html = "<h1>Article</h1><img src=\"test.jpg\" alt=\"test\">"
{markdown, metadata} = HtmlToMarkdown.convert_with_metadata(html)

IO.inspect(metadata.document.title)        # Document title
IO.inspect(metadata.headers)               # All h1-h6 elements
IO.inspect(metadata.links)                 # All hyperlinks
IO.inspect(metadata.images)                # All images with alt text
IO.inspect(metadata.structured_data)       # JSON-LD, Microdata, RDFa
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


```elixir
defmodule MyVisitor do
  def visit_link(ctx, href, text, title) do
    # Rewrite CDN URLs
    href = if String.starts_with?(href, "https://old-cdn.com") do
      String.replace(href, "https://old-cdn.com", "https://new-cdn.com")
    else
      href
    end
    {:custom, "[#{text}](#{href})"}
  end

  def visit_image(ctx, src, alt, title) do
    # Skip tracking pixels
    if String.contains?(src, "tracking") do
      :skip
    else
      :continue
    end
  end
end

html = "<a href=\"https://old-cdn.com/file.pdf\">Download</a>"
markdown = HtmlToMarkdown.convert_with_visitor(html, visitor: MyVisitor)
```



For comprehensive examples including content filtering, link footnotes, accessibility validation, and asynchronous URL validation, see the [Visitor Pattern Guide](../../examples/visitor-pattern/).



## Examples

- [Visitor Pattern Guide](../../examples/visitor-pattern/)
- [Metadata Extraction Guide](../../examples/metadata-extraction/)
- [Performance Guide](../../examples/performance/)

## Links

- **GitHub:** [github.com/kreuzberg-dev/html-to-markdown](https://github.com/kreuzberg-dev/html-to-markdown)

- **Hex.pm:** [hex.pm/packages/html_to_markdown](https://hex.pm/packages/html_to_markdown)

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
