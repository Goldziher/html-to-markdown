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

The visitor pattern allows you to customize HTML→Markdown conversion by providing callbacks for specific HTML elements. This enables fine-grained control over how individual elements are rendered.

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

### Visitor Context

Each visitor callback receives a `NodeContext` object with metadata about the current element:

```ruby
class DetailedVisitor
  def visit_link(ctx, href, text, title = nil)
    # ctx.node_type     => :link
    # ctx.tag_name      => "a"
    # ctx.attributes    => {"href" => "...", ...}
    # ctx.depth         => Integer
    # ctx.parent_tag    => String or nil
    # ctx.is_inline     => true/false

    { type: :continue }  # Use default conversion
  end
end
```

### Return Value Types

Each visitor callback must return a hash with at least a `:type` key. The following types are supported:

- **`:continue`** - Use default conversion for this element
  ```ruby
  { type: :continue }
  ```

- **`:custom`** - Replace with custom markdown output
  ```ruby
  { type: :custom, output: "[Click here](#{href})" }
  ```

- **`:skip`** - Omit this element entirely
  ```ruby
  { type: :skip }
  ```

- **`:preserve_html`** - Keep original HTML without conversion
  ```ruby
  { type: :preserve_html }
  ```

- **`:error`** - Stop conversion with an error message
  ```ruby
  { type: :error, message: "Unsupported element type" }
  ```

### Supported Visitor Methods

Implement any of these methods in your visitor class to customize specific elements:

**Element Lifecycle**
- `visit_element_start(ctx)` - Called when entering an element
- `visit_element_end(ctx, output)` - Called when leaving an element

**Content**
- `visit_text(ctx, text)` - Plain text nodes
- `visit_line_break(ctx)` - Line breaks (`<br>`)
- `visit_horizontal_rule(ctx)` - Horizontal rules (`<hr>`)

**Inline Formatting**
- `visit_link(ctx, href, text, title = nil)` - Hyperlinks
- `visit_image(ctx, src, alt, title = nil)` - Images
- `visit_code_inline(ctx, code)` - Inline code
- `visit_strong(ctx, text)` - Bold text
- `visit_emphasis(ctx, text)` - Italic text
- `visit_strikethrough(ctx, text)` - Strikethrough
- `visit_underline(ctx, text)` - Underlined text
- `visit_subscript(ctx, text)` - Subscript
- `visit_superscript(ctx, text)` - Superscript
- `visit_mark(ctx, text)` - Highlighted text

**Block Formatting**
- `visit_heading(ctx, level, text, id = nil)` - Headers (h1-h6)
- `visit_code_block(ctx, lang = nil, code)` - Code blocks
- `visit_blockquote(ctx, content, depth)` - Block quotes

**Lists**
- `visit_list_start(ctx, ordered)` - List opening
- `visit_list_item(ctx, ordered, marker, text)` - Individual list items
- `visit_list_end(ctx, ordered, output)` - List closing

**Tables**
- `visit_table_start(ctx)` - Table opening
- `visit_table_row(ctx, cells, is_header)` - Table rows
- `visit_table_end(ctx, output)` - Table closing

**Advanced Elements**
- `visit_definition_list_start(ctx)` - Definition list opening
- `visit_definition_term(ctx, text)` - Definition terms
- `visit_definition_description(ctx, text)` - Definition descriptions
- `visit_definition_list_end(ctx, output)` - Definition list closing
- `visit_figure_start(ctx)` - Figure opening
- `visit_figcaption(ctx, text)` - Figure captions
- `visit_figure_end(ctx, output)` - Figure closing
- `visit_form(ctx, action, method)` - Forms
- `visit_input(ctx, input_type, name, value)` - Form inputs
- `visit_button(ctx, text)` - Buttons
- `visit_details(ctx, open)` - Collapsible details
- `visit_summary(ctx, text)` - Summary elements
- `visit_audio(ctx, src)` - Audio elements
- `visit_video(ctx, src)` - Video elements
- `visit_iframe(ctx, src)` - Embedded iframes
- `visit_custom_element(ctx, tag_name, html)` - Unknown/custom elements

### Full Example

```ruby
require 'html_to_markdown'

class CustomVisitor
  def visit_heading(ctx, level, text, id = nil)
    # Add custom IDs to headings
    { type: :custom, output: "#{'#' * level} #{text} {#custom-#{id}}" }
  end

  def visit_link(ctx, href, text, title = nil)
    # Convert external links to footnotes
    if href.start_with?('http')
      { type: :custom, output: "#{text}[^1]" }
    else
      { type: :continue }
    end
  end

  def visit_code_block(ctx, lang = nil, code)
    # Wrap code blocks in custom markers
    marker = "```#{lang || 'text'}"
    { type: :custom, output: "#{marker}\n#{code}\n```" }
  end

  def visit_image(ctx, src, alt, title = nil)
    # Filter images by source
    if src.include?('ads')
      { type: :skip }  # Skip ad images
    else
      { type: :continue }
    end
  end
end

html = <<~HTML
  <h1 id="intro">Introduction</h1>
  <p>Check out <a href="https://example.com">this link</a>!</p>
  <pre><code class="language-ruby">puts "Hello"</code></pre>
  <img src="/ads/banner.jpg" alt="Ad">
  <img src="/content/diagram.jpg" alt="Diagram">
HTML

result = HtmlToMarkdown.convert_with_visitor(
  html,
  visitor: CustomVisitor.new,
  heading_style: :atx
)

puts result
```

### Type Safety with RBS

All visitor types are defined in `sig/html_to_markdown.rbs` for full type safety with Steep:

```ruby
# Type definitions available:
# - visitor_result - Return type for all visitor callbacks
# - NodeContext - Parameter type with node metadata

steep check  # Validate visitor implementation
```

Implement visitor callbacks with proper type hints to catch errors early:

```ruby
require 'html_to_markdown'

class TypedVisitor
  def visit_link(ctx : HtmlToMarkdown::NodeContext, href : String, text : String, title : String | nil = nil) : HtmlToMarkdown::visitor_result
    { type: :custom, output: "[#{text}](#{href})" }
  end
end
```

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
