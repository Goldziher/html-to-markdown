# html-to-markdown-rb

Blazing-fast HTML → Markdown conversion for Ruby, powered by the same Rust engine used by our Python, Node.js, WebAssembly, and PHP packages. Ship identical Markdown across every runtime while enjoying native extension performance.

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

## Features

- ⚡ **Rust-fast**: Ruby bindings around a highly optimised Rust core (60‑80× faster than BeautifulSoup-based converters).
- 🔁 **Identical output**: Shares logic with the Python wheels, npm bindings, PHP extension, WASM package, and CLI — consistent Markdown everywhere.
- ⚙️ **Rich configuration**: Control heading styles, list indentation, whitespace handling, HTML preprocessing, and more.
- 🖼️ **Inline image extraction**: Pull out embedded images (PNG/JPEG/SVG/data URIs) alongside Markdown.
- 🧰 **Bundled CLI proxy**: Call the Rust CLI straight from Ruby or shell scripts.
- 🛠️ **First-class Rails support**: Works with `Gem.win_platform?` builds, supports Trusted Publishing, and compiles on install if no native gem matches.

## Documentation & Support

- [GitHub repository](https://github.com/Goldziher/html-to-markdown)
- [Issue tracker](https://github.com/Goldziher/html-to-markdown/issues)
- [Changelog](https://github.com/Goldziher/html-to-markdown/blob/main/CHANGELOG.md)
- [Live demo (WASM)](https://goldziher.github.io/html-to-markdown/)

## Installation

```bash
bundle add html-to-markdown
# or
gem install html-to-markdown
```

Add the gem to your project and Bundler will compile the native Rust extension on first install.

### Requirements

- Ruby **3.2+** (Magnus relies on the fiber scheduler APIs added in 3.2)
- Rust toolchain **1.85+** with Cargo available on your `$PATH`
- Ruby development headers (`ruby-dev`, `ruby-devel`, or the platform equivalent)

**Windows**: install [RubyInstaller with MSYS2](https://rubyinstaller.org/) (UCRT64). Run once:

```powershell
ridk exec pacman -S --needed --noconfirm base-devel mingw-w64-ucrt-x86_64-toolchain
```

This provides the standard headers (including `strings.h`) required for the bindgen step.

## Performance Snapshot

Apple M4 • Real Wikipedia documents • `HtmlToMarkdown.convert` (Ruby)

| Document            | Size  | Latency | Throughput | Docs/sec |
| ------------------- | ----- | ------- | ---------- | -------- |
| Lists (Timeline)    | 129KB | 0.69ms  | 187 MB/s   | 1,450    |
| Tables (Countries)  | 360KB | 2.19ms  | 164 MB/s   | 456      |
| Mixed (Python wiki) | 656KB | 4.88ms  | 134 MB/s   | 205      |

> Same core, same benchmarks: the Ruby extension stays within single-digit % of the Rust CLI and mirrors the Python/Node numbers.

### Benchmark Fixtures (Apple M4)

Measured via `task bench:harness` with the shared Wikipedia + hOCR suite:

| Document               | Size   | ops/sec (Ruby) |
| ---------------------- | ------ | -------------- |
| Lists (Timeline)       | 129 KB | 3,156          |
| Tables (Countries)     | 360 KB | 921            |
| Medium (Python)        | 657 KB | 469            |
| Large (Rust)           | 567 KB | 534            |
| Small (Intro)          | 463 KB | 629            |
| hOCR German PDF        | 44 KB  | 7,250          |
| hOCR Invoice           | 4 KB   | 83,883         |
| hOCR Embedded Tables   | 37 KB  | 7,890          |

> These numbers line up with the Python/Node bindings because everything flows through the same Rust engine.

## Quick Start

```ruby
require 'html_to_markdown'

html = <<~HTML
  <h1>Welcome</h1>
  <p>This is <strong>Rust-fast</strong> conversion!</p>
  <ul>
    <li>Native extension</li>
    <li>Identical output across languages</li>
  </ul>
HTML

markdown = HtmlToMarkdown.convert(html)
puts markdown
# # Welcome
#
# This is **Rust-fast** conversion!
#
# - Native extension
# - Identical output across languages
```

## API

### Conversion Options

Pass a Ruby hash (string or symbol keys) to tweak rendering. Every option maps one-for-one with the Rust/Python/Node APIs.

```ruby
require 'html_to_markdown'

markdown = HtmlToMarkdown.convert(
  '<pre><code class="language-ruby">puts "hi"</code></pre>',
  heading_style: :atx,
  code_block_style: :fenced,
  bullets: '*+-',
  list_indent_type: :spaces,
  list_indent_width: 2,
  whitespace_mode: :normalized,
  highlight_style: :double_equal
)

puts markdown
```

### Reusing Options

If you’re running tight loops or benchmarks, build the options once and pass the handle back into `convert_with_options`:

```ruby
handle = HtmlToMarkdown.options(hocr_spatial_tables: false)

100.times do
  HtmlToMarkdown.convert_with_options('<h1>Handles</h1>', handle)
end
```

### HTML Preprocessing

Clean up scraped HTML (navigation, forms, malformed markup) before conversion:

```ruby
require 'html_to_markdown'

markdown = HtmlToMarkdown.convert(
  html,
  preprocessing: {
    enabled: true,
    preset: :aggressive, # :minimal, :standard, :aggressive
    remove_navigation: true,
    remove_forms: true
  }
)
```

### Inline Images

Extract inline binary data (data URIs, SVG) together with the converted Markdown.

```ruby
require 'html_to_markdown'

result = HtmlToMarkdown.convert_with_inline_images(
  '<img src="data:image/png;base64,iVBORw0..." alt="Pixel">',
  image_config: {
    max_decoded_size_bytes: 1 * 1024 * 1024,
    infer_dimensions: true,
    filename_prefix: 'img_',
    capture_svg: true
  }
)

puts result.markdown
result.inline_images.each do |img|
  puts "#{img.filename} -> #{img.format} (#{img.data.bytesize} bytes)"
end
```

### Metadata Extraction

Extract comprehensive metadata alongside Markdown conversion: document properties (title, description, author, language), social metadata (Open Graph, Twitter cards), heading hierarchy, link analysis (type classification, rel attributes), image metadata (dimensions, type detection), and structured data (JSON-LD, Microdata, RDFa).

#### Basic Usage

```ruby
require 'html_to_markdown'

html = '<html lang="en"><head><title>Test</title></head><body><h1>Hello</h1></body></html>'
markdown, metadata = HtmlToMarkdown.convert_with_metadata(html)

puts markdown
puts metadata[:document][:title]  # "Test"
puts metadata[:headers].length     # 1
```

#### With Conversion Options

```ruby
conv_opts = { heading_style: :atx_closed }
metadata_opts = { extract_headers: true, extract_links: false }

markdown, metadata = HtmlToMarkdown.convert_with_metadata(
  html,
  conv_opts,
  metadata_opts
)
```

#### Full Example

```ruby
require 'html_to_markdown'

html = <<~HTML
  <html>
    <head>
      <title>Example</title>
      <meta name="description" content="Demo page">
      <link rel="canonical" href="https://example.com/page">
      <meta property="og:image" content="https://example.com/og.jpg">
      <meta name="twitter:card" content="summary_large_image">
    </head>
    <body>
      <h1 id="welcome">Welcome</h1>
      <a href="https://example.com" rel="nofollow external">Example link</a>
      <img src="https://example.com/image.jpg" alt="Hero" width="640" height="480">
      <script type="application/ld+json">
        {"@context": "https://schema.org", "@type": "Article"}
      </script>
    </body>
  </html>
HTML

markdown, metadata = HtmlToMarkdown.convert_with_metadata(
  html,
  { heading_style: :atx },
  { extract_links: true, extract_images: true, extract_headers: true, extract_structured_data: true }
)

puts markdown
puts metadata[:document][:title]         # "Example"
puts metadata[:document][:description]   # "Demo page"
puts metadata[:document][:open_graph]    # {"og:image" => "https://example.com/og.jpg"}
puts metadata[:links].first[:rel]        # ["nofollow", "external"]
puts metadata[:images].first[:dimensions] # [640, 480]
puts metadata[:headers].first[:id]       # "welcome"
```

#### Return Value Structure

Returns a 2-element array: `[markdown_string, metadata_hash]`

The metadata hash contains:

```ruby
{
  document: {
    title: String?,
    description: String?,
    keywords: Array[String],
    author: String?,
    canonical_url: String?,
    base_href: String?,
    language: String?,
    text_direction: "ltr" | "rtl" | "auto" | nil,
    open_graph: Hash[String, String],
    twitter_card: Hash[String, String],
    meta_tags: Hash[String, String]
  },
  headers: [
    {
      level: Integer,          # 1-6
      text: String,
      id: String?,
      depth: Integer,
      html_offset: Integer
    }
  ],
  links: [
    {
      href: String,
      text: String,
      title: String?,
      link_type: "anchor" | "internal" | "external" | "email" | "phone" | "other",
      rel: Array[String],
      attributes: Hash[String, String]
    }
  ],
  images: [
    {
      src: String,
      alt: String?,
      title: String?,
      dimensions: [Integer, Integer]?,
      image_type: "data_uri" | "inline_svg" | "external" | "relative",
      attributes: Hash[String, String]
    }
  ],
  structured_data: [
    {
      data_type: "json_ld" | "microdata" | "rdfa",
      raw_json: String,
      schema_type: String?
    }
  ]
}
```

#### Metadata Configuration

Pass a hash with the following options to control which metadata types are extracted:

```ruby
config = {
  extract_headers: true,           # Extract h1-h6 elements (default: true)
  extract_links: true,             # Extract <a> elements (default: true)
  extract_images: true,            # Extract <img> elements (default: true)
  extract_structured_data: true,   # Extract JSON-LD/Microdata/RDFa (default: true)
  max_structured_data_size: 1_000_000  # Max bytes for structured data (default: 1MB)
}

markdown, metadata = HtmlToMarkdown.convert_with_metadata(html, nil, config)
```

#### Features

The Ruby binding provides comprehensive metadata extraction during HTML-to-Markdown conversion:

- **Document Metadata**: title, description, keywords, author, canonical URL, language, text direction
- **Open Graph & Twitter Card**: social media metadata extraction
- **Headers**: h1-h6 extraction with hierarchy, ids, and depth tracking
- **Links**: hyperlink extraction with type classification (anchor, internal, external, email, phone)
- **Images**: image extraction with source type (data_uri, inline_svg, external, relative) and dimensions
- **Structured Data**: JSON-LD, Microdata, and RDFa extraction

#### Type Safety with RBS

All types are defined in RBS format in `sig/html_to_markdown.rbs`:

- `document_metadata` - Document-level metadata structure
- `header_metadata` - Individual header element
- `link_metadata` - Individual link element
- `image_metadata` - Individual image element
- `structured_data` - Structured data block
- `extended_metadata` - Complete metadata extraction result

Uses strict RBS type checking with Steep for full type safety:

```bash
steep check
```

#### Implementation Architecture

The Rust implementation uses a single-pass collector pattern for efficient metadata extraction:

1. **No duplication**: Core logic lives in Rust (`crates/html-to-markdown/src/metadata.rs`)
2. **Minimal wrapper layer**: Ruby binding in `crates/html-to-markdown-rb/src/lib.rs`
3. **Type translation**: Rust types → Ruby hashes with proper Magnus bindings
4. **Hash conversion**: Uses Magnus `RHash` API for efficient Ruby hash construction

The metadata feature is gated by a Cargo feature in `Cargo.toml`:

```toml
[features]
metadata = ["html-to-markdown-rs/metadata"]
```

This ensures:
- Zero overhead when metadata is not needed
- Clean integration with feature flag detection
- Consistent with Python binding implementation

#### Language Parity

Implements the same API as the Python binding:

- Same method signature: `convert_with_metadata(html, options, metadata_config)`
- Same return type: `[markdown, metadata_dict]`
- Same metadata structures and field names
- Same enum values (link_type, image_type, data_type, text_direction)

Enables seamless migration and multi-language development.

#### Performance

Single-pass collection during tree traversal:
- No additional parsing passes
- Minimal memory overhead
- Configurable extraction granularity
- Built-in size limits for safety

#### Testing

Comprehensive RSpec test suite in `spec/metadata_extraction_spec.rb`:

```bash
cd packages/ruby
bundle exec rake compile -- --release --features metadata
bundle exec rspec spec/metadata_extraction_spec.rb
```

Tests cover:
- All metadata types extraction
- Configuration flags
- Edge cases (empty HTML, malformed input, special characters)
- Return value structure validation
- Integration with conversion options

## Visitor Pattern

The visitor pattern allows you to customize HTML→Markdown conversion by providing callbacks for specific HTML elements. This enables fine-grained control over how individual elements are rendered without duplicating conversion logic.

Ruby visitors are **synchronous** — all 40+ visitor methods execute in the same call stack during tree traversal, making it easy to maintain state across multiple elements.

### Overview

The visitor pattern is useful for:
- Custom element handling (e.g., stripping ads, rewriting links)
- Domain-specific markdown dialects (e.g., footnotes, custom admonition syntax)
- Analytics or extraction during conversion (e.g., counting headings, collecting metadata)
- Conditional rendering based on element context

### Basic Example

```ruby
require 'html_to_markdown'

class MyVisitor
  def visit_link(ctx, href, text, title = nil)
    # Customize link conversion
    { type: :custom, output: "[#{text}](#{href})" }
  end

  def visit_image(ctx, src, alt, title = nil)
    # Skip all images
    { type: :skip }
  end
end

html = '<a href="/page">Link</a><img src="pic.jpg">'
result = HtmlToMarkdown.convert_with_visitor(html, visitor: MyVisitor.new)
puts result
# => [Link](/page)
```

### NodeContext: Element Metadata

Each visitor callback receives a `NodeContext` object with comprehensive metadata about the current element:

```ruby
class DetailedVisitor
  def visit_link(ctx, href, text, title = nil)
    # ctx.node_type     => :link (Symbol identifier for element type)
    # ctx.tag_name      => "a" (HTML tag name)
    # ctx.attributes    => {"href" => "...", "class" => "..."}
    # ctx.depth         => Integer (nesting depth from root)
    # ctx.index_in_parent => Integer (position among siblings)
    # ctx.parent_tag    => "p" (tag of parent element, or nil)
    # ctx.is_inline     => true/false (inline vs block context)

    { type: :continue }  # Use default conversion
  end
end
```

Use context information to make intelligent decisions:

```ruby
class SmartVisitor
  def visit_link(ctx, href, text, title = nil)
    # Only rewrite external links
    if href.start_with?('http')
      { type: :custom, output: "[#{text}](#{href}){.external}" }
    else
      # Keep internal links as-is
      { type: :continue }
    end
  end

  def visit_heading(ctx, level, text, id = nil)
    # Only add custom styling to top-level headings
    if ctx.depth == 1 && level == 1
      { type: :custom, output: "#{'#' * level} #{text} {.title}" }
    else
      { type: :continue }
    end
  end
end
```

### VisitResult: Return Types

Every visitor callback must return a hash with a `:type` key. The supported result types are:

#### Continue (Use Default Behavior)
```ruby
def visit_link(ctx, href, text, title = nil)
  { type: :continue }  # Rust engine handles conversion
end
```
Tells the converter to use its default behavior for this element. Useful when you want to skip certain elements but keep the majority unchanged.

#### Custom (Replace with Custom Output)
```ruby
def visit_heading(ctx, level, text, id = nil)
  { type: :custom, output: "#{'#' * level} #{text}" }
end
```
Replace the element's markdown with your custom output string. You're responsible for generating valid markdown.

#### Skip (Omit Element)
```ruby
def visit_image(ctx, src, alt, title = nil)
  # Remove all images from output
  { type: :skip }
end
```
Removes the element entirely from the output. Useful for filtering unwanted content (ads, tracking pixels, etc.).

#### Preserve HTML (Keep Original HTML)
```ruby
def visit_custom_element(ctx, tag_name, html)
  # Keep unknown HTML as-is
  { type: :preserve_html }
end
```
Leaves the element in its original HTML form without markdown conversion. Useful for preserving custom HTML that markdown doesn't support.

#### Error (Stop Conversion)
```ruby
def visit_input(ctx, input_type, name, value)
  # Stop conversion if we encounter forms
  { type: :error, message: "Forms are not supported" }
end
```
Halts conversion immediately and raises `HtmlToMarkdown::Error` with your message. Use this for fail-fast validation.

### Supported Visitor Methods

Implement any of these 40+ methods in your visitor class:

**Element Lifecycle** (called on all elements)
- `visit_element_start(ctx)` → `visitor_result` - Called when entering any element
- `visit_element_end(ctx, output)` → `visitor_result` - Called when leaving any element

**Text Content**
- `visit_text(ctx, text)` → `visitor_result` - Plain text nodes
- `visit_line_break(ctx)` → `visitor_result` - Line breaks (`<br>`)
- `visit_horizontal_rule(ctx)` → `visitor_result` - Horizontal rules (`<hr>`)

**Inline Formatting** (text-level elements)
- `visit_link(ctx, href, text, title = nil)` → `visitor_result` - Hyperlinks (`<a>`)
- `visit_image(ctx, src, alt, title = nil)` → `visitor_result` - Images (`<img>`)
- `visit_code_inline(ctx, code)` → `visitor_result` - Inline code (`<code>`)
- `visit_strong(ctx, text)` → `visitor_result` - Bold text (`<strong>`, `<b>`)
- `visit_emphasis(ctx, text)` → `visitor_result` - Italic text (`<em>`, `<i>`)
- `visit_strikethrough(ctx, text)` → `visitor_result` - Strikethrough (`<del>`, `<s>`)
- `visit_underline(ctx, text)` → `visitor_result` - Underlined text (`<u>`)
- `visit_subscript(ctx, text)` → `visitor_result` - Subscript (`<sub>`)
- `visit_superscript(ctx, text)` → `visitor_result` - Superscript (`<sup>`)
- `visit_mark(ctx, text)` → `visitor_result` - Highlighted text (`<mark>`)

**Block Formatting** (block-level elements)
- `visit_heading(ctx, level, text, id = nil)` → `visitor_result` - Headers (`<h1>`-`<h6>`)
- `visit_code_block(ctx, lang = nil, code)` → `visitor_result` - Code blocks (`<pre><code>`)
- `visit_blockquote(ctx, content, depth)` → `visitor_result` - Block quotes (`<blockquote>`)

**Lists**
- `visit_list_start(ctx, ordered)` → `visitor_result` - List opening (`<ul>`, `<ol>`)
- `visit_list_item(ctx, ordered, marker, text)` → `visitor_result` - List items (`<li>`)
- `visit_list_end(ctx, ordered, output)` → `visitor_result` - List closing

**Tables**
- `visit_table_start(ctx)` → `visitor_result` - Table opening (`<table>`)
- `visit_table_row(ctx, cells, is_header)` → `visitor_result` - Table rows (`<tr>`)
- `visit_table_end(ctx, output)` → `visitor_result` - Table closing

**Advanced Elements**
- `visit_definition_list_start(ctx)` → `visitor_result` - Definition list opening (`<dl>`)
- `visit_definition_term(ctx, text)` → `visitor_result` - Definition terms (`<dt>`)
- `visit_definition_description(ctx, text)` → `visitor_result` - Definition descriptions (`<dd>`)
- `visit_definition_list_end(ctx, output)` → `visitor_result` - Definition list closing
- `visit_figure_start(ctx)` → `visitor_result` - Figure opening (`<figure>`)
- `visit_figcaption(ctx, text)` → `visitor_result` - Figure captions (`<figcaption>`)
- `visit_figure_end(ctx, output)` → `visitor_result` - Figure closing
- `visit_form(ctx, action, method)` → `visitor_result` - Forms (`<form>`)
- `visit_input(ctx, input_type, name, value)` → `visitor_result` - Form inputs (`<input>`)
- `visit_button(ctx, text)` → `visitor_result` - Buttons (`<button>`)
- `visit_details(ctx, open)` → `visitor_result` - Collapsible details (`<details>`)
- `visit_summary(ctx, text)` → `visitor_result` - Summary elements (`<summary>`)
- `visit_audio(ctx, src)` → `visitor_result` - Audio elements (`<audio>`)
- `visit_video(ctx, src)` → `visitor_result` - Video elements (`<video>`)
- `visit_iframe(ctx, src)` → `visitor_result` - Embedded iframes (`<iframe>`)
- `visit_custom_element(ctx, tag_name, html)` → `visitor_result` - Unknown/custom elements

### Practical Examples

#### Custom Image Handling

Filter and rewrite images based on source:

```ruby
class ImageFilterVisitor
  attr_reader :images

  def initialize
    @images = []
  end

  def visit_image(ctx, src, alt, title = nil)
    # Skip tracking pixels
    return { type: :skip } if src.include?('tracking') || src.include?('beacon')

    # Rewrite CDN URLs
    if src.start_with?('//cdn.example.com')
      rewritten = src.sub('//cdn.example.com', 'https://cdn.local')
      @images << { src: rewritten, alt: alt }
      { type: :custom, output: "![#{alt}](#{rewritten})" }
    else
      @images << { src: src, alt: alt }
      { type: :continue }
    end
  end
end

html = '<img src="//cdn.example.com/image.jpg" alt="Logo"><img src="/tracking/pixel.gif">'
visitor = ImageFilterVisitor.new
result = HtmlToMarkdown.convert_with_visitor(html, visitor: visitor)

puts result
# => ![Logo](https://cdn.local/image.jpg)

puts "Collected images: #{visitor.images.inspect}"
# => [{:src=>"https://cdn.local/image.jpg", :alt=>"Logo"}]
```

#### Link Modification and Analytics

Track link types and rewrite external links:

```ruby
class LinkAnalyticsVisitor
  attr_reader :stats

  def initialize
    @stats = { internal: 0, external: 0, email: 0, total_links: [] }
  end

  def visit_link(ctx, href, text, title = nil)
    @stats[:total_links] << href

    if href.start_with?('mailto:')
      @stats[:email] += 1
      { type: :continue }
    elsif href.start_with?('http')
      @stats[:external] += 1
      # Add tracking parameter to external links
      tracked = "#{href}#{href.include?('?') ? '&' : '?'}utm_source=markdown"
      { type: :custom, output: "[#{text}](#{tracked})" }
    else
      @stats[:internal] += 1
      { type: :continue }
    end
  end
end

html = <<~HTML
  <a href="/home">Home</a>
  <a href="https://example.com">External</a>
  <a href="mailto:test@example.com">Email</a>
HTML

visitor = LinkAnalyticsVisitor.new
result = HtmlToMarkdown.convert_with_visitor(html, visitor: visitor)

puts "Link stats: #{visitor.stats.inspect}"
# => {:internal=>1, :external=>1, :email=>1, :total_links=>["/home", "https://example.com", "mailto:test@example.com"]}
```

#### Custom Markdown Dialect (Footnotes)

Convert external links to a footnote-based dialect:

```ruby
class FootnoteVisitor
  attr_reader :footnotes

  def initialize
    @footnotes = []
  end

  def visit_link(ctx, href, text, title = nil)
    if href.start_with?('http')
      # External links become footnotes
      fn_num = @footnotes.length + 1
      @footnotes << { num: fn_num, href: href, title: title }
      { type: :custom, output: "#{text}[^#{fn_num}]" }
    else
      # Internal links stay as links
      { type: :continue }
    end
  end

  def footnotes_section
    return "" if @footnotes.empty?

    lines = ["\n\n---\n\n"]
    @footnotes.each do |fn|
      lines << "[^#{fn[:num]}]: #{fn[:href]}"
      lines << " \"#{fn[:title]}\"" if fn[:title]
      lines << "\n"
    end
    lines.join
  end
end

html = <<~HTML
  <p>Check out <a href="https://example.com">this site</a> and <a href="https://github.com">GitHub</a>.</p>
  <p><a href="/local/page">Local link</a> stays unchanged.</p>
HTML

visitor = FootnoteVisitor.new
result = HtmlToMarkdown.convert_with_visitor(html, visitor: visitor)

puts result + visitor.footnotes_section
# => Check out this site[^1] and GitHub[^2].
#
#    Local link stays unchanged.
#
#    ---
#
#    [^1]: https://example.com
#    [^2]: https://github.com
```

### Type Safety with RBS

All visitor types are defined in `sig/html_to_markdown.rbs` for full type safety with [Steep](https://github.com/soutaro/steep):

```ruby
# Type definitions available:
# - HtmlToMarkdown::visitor_result - Return type for all callbacks
# - HtmlToMarkdown::NodeContext - Parameter type with element metadata

steep check  # Validate your visitor implementation
```

Implement callbacks with full type annotations to catch errors at type-check time:

```ruby
require 'html_to_markdown'

class TypedVisitor
  def visit_link(
    ctx : HtmlToMarkdown::NodeContext,
    href : String,
    text : String,
    title : String | nil = nil
  ) : HtmlToMarkdown::visitor_result
    { type: :custom, output: "[#{text}](#{href})" }
  end

  def visit_image(
    ctx : HtmlToMarkdown::NodeContext,
    src : String,
    alt : String | nil,
    title : String | nil = nil
  ) : HtmlToMarkdown::visitor_result
    # Return type is validated against visitor_result union
    { type: :skip }
  end
end

# Type check your visitor
# $ steep check
```

Type safety ensures:
- All callback signatures match RBS definitions
- Return types are valid `visitor_result` variants
- Context parameter types are correct
- Early detection of mistakes via LSP in your editor

## CLI

The gem bundles a small proxy for the Rust CLI binary. Use it when you need parity with the standalone `html-to-markdown` executable.

```ruby
require 'html_to_markdown/cli'

HtmlToMarkdown::CLI.run(%w[--heading-style atx input.html], stdout: $stdout)
# => writes converted Markdown to STDOUT
```

You can also call the CLI binary directly for scripting:

```ruby
HtmlToMarkdown::CLIProxy.call(['--version'])
# => "html-to-markdown 2.5.7"
```

Rebuild the CLI locally if you see `CLI binary not built` during tests:

```bash
bundle exec rake compile          # builds the extension
bundle exec ruby scripts/prepare_ruby_gem.rb  # copies the CLI into lib/bin/
```

## Error Handling

Conversion errors raise `HtmlToMarkdown::Error` (wrapping the Rust error context). CLI invocations use specialised subclasses:

- `HtmlToMarkdown::CLIProxy::MissingBinaryError`
- `HtmlToMarkdown::CLIProxy::CLIExecutionError`

Rescue them to provide clearer feedback in your application.

Inputs that look like binary data (e.g., PDF bytes coerced to a string) raise `HtmlToMarkdown::Error` with an
`Invalid input` message.

## Consistent Across Languages

The Ruby gem shares the exact Rust core with:

- [Python wheels](https://pypi.org/project/html-to-markdown/)
- [Node.js / Bun bindings](https://www.npmjs.com/package/html-to-markdown-node)
- [WebAssembly package](https://www.npmjs.com/package/html-to-markdown-wasm)
- The Rust crate and CLI

Use whichever runtime fits your stack while keeping formatting behaviour identical.

## Development

```bash
bundle exec rake compile   # build the native extension
bundle exec rspec          # run test suite
```

The extension uses [Magnus](https://github.com/matsadler/magnus) plus `rb-sys` for bindgen. When editing the Rust code under `src/`, rerun `rake compile`.

## License

MIT © Na'aman Hirschfeld
