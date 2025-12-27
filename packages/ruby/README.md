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

## API Reference

### Basic Conversion

```ruby
# Simple conversion
markdown = HtmlToMarkdown.convert(html)

# With options (pass a Ruby hash with symbol keys)
markdown = HtmlToMarkdown.convert(html, heading_style: :atx, code_block_style: :fenced)

# With inline images
result = HtmlToMarkdown.convert_with_inline_images(html, image_config: {...})
markdown = result.markdown
images = result.inline_images

# With metadata extraction
markdown, metadata = HtmlToMarkdown.convert_with_metadata(html, options, metadata_config)

# With visitor pattern (custom callbacks)
result = HtmlToMarkdown.convert_with_visitor(html, visitor: MyVisitor.new, options: {...})
```

### Conversion Options Hash

```ruby
{
  heading_style: :atx,                    # :atx or :setext
  code_block_style: :fenced,              # :fenced or :indented
  bullets: '*+-',                         # List bullet chars
  list_indent_type: :spaces,              # :spaces or :tabs
  list_indent_width: 2,                   # Number of indent spaces
  whitespace_mode: :normalized,           # :normalized, :preserve, or :collapse
  highlight_style: :double_equal,         # Code highlighting style
  hocr_spatial_tables: false,             # Special hOCR table handling
  preprocessing: {
    enabled: true,
    preset: :aggressive,                  # :minimal, :standard, :aggressive
    remove_navigation: true,
    remove_forms: true
  }
}
```

### Performance: Reusing Options

For tight loops, build an options handle once:

```ruby
handle = HtmlToMarkdown.options(hocr_spatial_tables: false)

100.times do
  HtmlToMarkdown.convert_with_options(html, handle)
end
```

### Metadata Extraction

Extract document properties (title, description, author, language), social metadata (Open Graph, Twitter cards), heading hierarchy, link analysis, image metadata, and structured data (JSON-LD, Microdata, RDFa):

```ruby
html = '<html lang="en"><head><title>Test</title></head><body><h1>Hello</h1></body></html>'
markdown, metadata = HtmlToMarkdown.convert_with_metadata(html)

puts metadata[:document][:title]     # "Test"
puts metadata[:headers].first[:text] # "Hello"
```

For detailed examples (SEO extraction, heading hierarchy analysis, structured data) and full metadata structure reference, see [Metadata Extraction Guide](../../examples/metadata-extraction/).

### Visitor Pattern

Customize conversion with fine-grained element callbacks. Perfect for custom element handling, analytics during conversion, domain-specific markdown dialects, and conditional rendering:

```ruby
class MyVisitor
  def visit_link(ctx, href, text, title = nil)
    { type: :custom, output: "[#{text}](#{href})" }
  end

  def visit_image(ctx, src, alt, title = nil)
    { type: :skip }  # Remove images
  end
end

result = HtmlToMarkdown.convert_with_visitor(html, visitor: MyVisitor.new)
```

**Return types**: `{ type: :continue }` (default), `{ type: :custom, output: "..." }` (replace), `{ type: :skip }` (omit), `{ type: :preserve_html }` (keep HTML), `{ type: :error, message: "..." }` (halt).

**40+ visitor methods** for text, inline formatting, blocks, lists, tables, advanced elements, and lifecycle hooks. Callback parameters include `NodeContext` with element metadata (tag_name, attributes, depth, parent_tag, is_inline).

For advanced examples (image filtering, link analytics, footnote dialects), RBS type-safety patterns, and full method reference, see [Visitor Pattern Guide](../../examples/visitor-pattern/).

## RBS Types & Strict Type Checking

Full RBS type definitions in `sig/html_to_markdown.rbs` enable strict type checking with [Steep](https://github.com/soutaro/steep):

```bash
steep check
```

Key types:
- `HtmlToMarkdown::NodeContext` - Element metadata in visitor callbacks (tag_name, attributes, depth, etc.)
- `HtmlToMarkdown::visitor_result` - Return type union for visitor methods
- `HtmlToMarkdown::extended_metadata` - Metadata extraction result

Type-safe visitor implementation:

```ruby
class TypedVisitor
  def visit_link(
    ctx : HtmlToMarkdown::NodeContext,
    href : String,
    text : String,
    title : String | nil = nil
  ) : HtmlToMarkdown::visitor_result
    { type: :custom, output: "[#{text}](#{href})" }
  end
end
```

All public methods are typed for early error detection and LSP editor support (Ruby 3+).

## Magnus Native Extension

The gem compiles a native Rust extension via [Magnus](https://github.com/matsadler/magnus) FFI bindings:

- **Zero-copy interop**: String and hash data flows directly between Ruby and Rust
- **Safe bindings**: No segfaults; Rust's type system ensures memory safety
- **Automatic error mapping**: Rust errors convert to Ruby exceptions with full context
- **Native performance**: Compiled to `.so` (Linux/macOS) or `.dll` (Windows)
- **Smart compilation**: Prebuilt binaries for common platforms; falls back to on-install compilation

Build manually:

```bash
bundle exec rake compile
```

## CLI Proxy

Call the Rust CLI directly from Ruby or shell:

```ruby
require 'html_to_markdown/cli'

HtmlToMarkdown::CLI.run(%w[--heading-style atx input.html], stdout: $stdout)

# Or call the binary directly
HtmlToMarkdown::CLIProxy.call(['--version'])
```

## Error Handling

- `HtmlToMarkdown::Error` - Conversion errors with Rust error context
- `HtmlToMarkdown::CLIProxy::MissingBinaryError` - CLI binary not found
- `HtmlToMarkdown::CLIProxy::CLIExecutionError` - Command execution failed

Binary data inputs (e.g., PDF bytes as string) raise `HtmlToMarkdown::Error` with "Invalid input" message.

## Examples

Comprehensive guides with real-world patterns (Ruby examples included):

- **[Visitor Pattern](../../examples/visitor-pattern/)** - Custom callbacks, element-by-element control, analytics, domain-specific markdown dialects
- **[Metadata Extraction](../../examples/metadata-extraction/)** - SEO data, heading hierarchy, link classification, structured data parsing
- **[Performance Guide](../../examples/performance/)** - Benchmarking, profiling, throughput optimization

## Consistent Across Languages

The Ruby gem shares the exact Rust core with:

- [Python wheels](https://pypi.org/project/html-to-markdown/)
- [Node.js / Bun bindings](https://www.npmjs.com/package/html-to-markdown-node)
- [WebAssembly package](https://www.npmjs.com/package/html-to-markdown-wasm)
- [PHP extension](https://packagist.org/packages/goldziher/html-to-markdown)
- The Rust crate and CLI

Use whichever runtime fits your stack while keeping formatting behaviour identical.

## Development

```bash
bundle exec rake compile   # build the native extension
bundle exec rspec          # run test suite
```

When editing Rust code under `src/`, rerun `rake compile`.

## License

MIT © Na'aman Hirschfeld
