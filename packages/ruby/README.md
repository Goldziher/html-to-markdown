# html-to-markdown-rb

Blazing-fast HTML → Markdown conversion for Ruby, powered by the same Rust engine used by our Python, Node.js, WebAssembly, and PHP packages. Ship identical Markdown across every runtime while enjoying native extension performance.

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

Measured via `task bench:bindings -- --language ruby` with the shared Wikipedia + hOCR suite:

| Document               | Size   | ops/sec (Ruby) |
| ---------------------- | ------ | -------------- |
| Lists (Timeline)       | 129 KB | 1,349          |
| Tables (Countries)     | 360 KB | 326            |
| Medium (Python)        | 657 KB | 157            |
| Large (Rust)           | 567 KB | 174            |
| Small (Intro)          | 463 KB | 214            |
| hOCR German PDF        | 44 KB  | 2,936          |
| hOCR Invoice           | 4 KB   | 25,740         |
| hOCR Embedded Tables   | 37 KB  | 3,328          |

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
