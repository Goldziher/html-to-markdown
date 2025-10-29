# html-to-markdown-rb

Ruby bindings for the `html-to-markdown` Rust engine – the same core that powers the Python wheels, Node.js NAPI bindings, WebAssembly package, and CLI. The gem exposes fast HTML → Markdown conversion with identical rendering behaviour across every supported language.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg)](https://crates.io/crates/html-to-markdown-rs)
[![npm version](https://badge.fury.io/js/html-to-markdown-node.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Gem Version](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)

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

### Conversion with Options

All configuration mirrors the Rust API. Options accept symbols or strings and match the same defaults as the other bindings.

```ruby
require 'html_to_markdown'

markdown = HtmlToMarkdown.convert(
  '<pre><code class="language-ruby">puts "hi"</code></pre>',
  heading_style: :atx,
  code_block_style: :fenced,
  bullets: ['*', '-', '+'],
  wrap: true,
  wrap_width: 80,
  preserve_tags: %w[table figure]
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

### CLI Proxy

The gem bundles a small proxy for the Rust CLI binary. Use it when you need parity with the standalone `html-to-markdown` executable.

```ruby
require 'html_to_markdown/cli'

HtmlToMarkdown::CLI.run(%w[--heading-style atx input.html], stdout: $stdout)
# => writes converted Markdown to STDOUT
```

You can also call the CLI binary directly for scripting:

```ruby
HtmlToMarkdown::CLIProxy.call(['--version'])
# => "html-to-markdown 2.5.3"
```

### Error Handling

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
