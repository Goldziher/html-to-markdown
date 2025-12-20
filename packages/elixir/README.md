# HtmlToMarkdown (Elixir)

Elixir bindings for the Rust [html-to-markdown](https://github.com/Goldziher/html-to-markdown) engine.
The package exposes a fast `HTML -> Markdown` converter implemented with Rustler.

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

Add `:html_to_markdown` to your `mix.exs` dependencies:

```elixir
def deps do
  [
    {:html_to_markdown, "~> 2.8"}
  ]
end
```

Compile the NIF (Rust and cargo are required):

```
mix deps.get
mix compile
```

## Prerequisites

- Elixir **1.19+** running on **OTP 28** (matches CI + release automation targets)
- Rust toolchain (stable) with `cargo` available

## Usage

```elixir
alias HtmlToMarkdown.{InlineImageConfig, Options}

iex> {:ok, markdown} = HtmlToMarkdown.convert("<h1>Hello</h1>")
iex> markdown
"# Hello\n"

iex> HtmlToMarkdown.convert!("<p>Example</p>", wrap: true, wrap_width: 20)
"Example\n"

# Pre-build reusable options
iex> handle = HtmlToMarkdown.options(%Options{wrap: true, wrap_width: 40})
iex> HtmlToMarkdown.convert_with_options("<p>Reusable</p>", handle)
{:ok, "Reusable\n"}
```

Supported options mirror the Rust `ConversionOptions` structure and are exposed
via the `%HtmlToMarkdown.Options{}` struct (or plain maps/keyword lists). Key
fields include:

- `heading_style`, `list_indent_type`, `newline_style`, `code_block_style` – atom
  values (`:atx`, `:tabs`, `:spaces`, etc.) mirroring the Rust enums.
- `wrap` / `wrap_width` – enable CommonMark soft breaks and configure the column
  width.
- `keep_inline_images_in`, `strip_tags`, `preserve_tags` – map sets or lists of
  tag names that control special handling for certain nodes.
- `preprocessing` – nested `%HtmlToMarkdown.PreprocessingOptions{}` (or maps)
  that toggles `:preset`, `:remove_forms`, `:remove_navigation`, etc.
- `debug` – turns on verbose tracing from the Rust core.

### Inline image extraction

`convert_with_inline_images/3` returns Markdown plus decoded image blobs and
warnings emitted during extraction:

```elixir
html = ~S(<p><img src="data:image/png;base64,..." alt="Logo"></p>)
config = %InlineImageConfig{infer_dimensions: true}

{:ok, markdown, inline_images, warnings} =
  HtmlToMarkdown.convert_with_inline_images(html, %{wrap: false}, config)

Enum.each(inline_images, fn image ->
  File.write!("output/#{image.filename}", image.data)
end)
```

`InlineImageConfig` can be built from a struct, map, or keyword list and accepts
`max_decoded_size_bytes`, `filename_prefix`, `capture_svg`, and
`infer_dimensions`. Invalid configs return `{:error, reason}` before any native
code runs.

Inline images are returned as `%HtmlToMarkdown.InlineImage{}` structs with the
following fields:

- `data` – raw bytes decoded from the `<img>` or inline `<svg>`.
- `format` – subtype string (for example `"png"` or `"svg"`).
- `filename` / `description` – optional DOM metadata.
- `dimensions` – `{width, height}` tuple when dimension inference is enabled.
- `source` – `"img_data_uri"` or `"svg_element"` indicating where the payload
  originated.
- `attributes` – remaining DOM attributes preserved as a map.

Warnings are exposed as `%HtmlToMarkdown.InlineImageWarning{index, message}`;
use `index` to correlate warnings back to the zero-based position in the inline
image list.

### Metadata extraction

`convert_with_metadata/3` returns Markdown plus a metadata map:

```elixir
html = """
<html>
  <head>
    <title>Example</title>
    <meta name="description" content="Demo page">
  </head>
  <body>
    <h1 id="welcome">Welcome</h1>
    <a href="https://example.com" rel="nofollow external">Example link</a>
  </body>
</html>
"""

{:ok, markdown, metadata} = HtmlToMarkdown.convert_with_metadata(html)

metadata["document"]["title"]        # "Example"
metadata["headers"] |> hd() |> Map.get("text") # "Welcome"
metadata["links"]   |> hd() |> Map.get("link_type") # "external"
```

## Performance (Apple M4)

Benchmarks use the shared Wikipedia + hOCR fixtures from the benchmark harness
in `tools/benchmark-harness`.

| Document               | Size   | Ops/sec | Throughput |
| ---------------------- | ------ | ------- | ---------- |
| Lists (Timeline)       | 129 KB | 1,463   | 184.8 MB/s |
| Tables (Countries)     | 360 KB |   357   | 125.5 MB/s |
| Medium (Python)        | 656 KB |   171   | 109.9 MB/s |
| Large (Rust)           | 567 KB |   174   |  96.4 MB/s |
| Small (Intro)          | 463 KB |   247   | 111.9 MB/s |
| HOCR German PDF        |  44 KB | 3,113   | 132.8 MB/s |
| HOCR Embedded Tables   |  37 KB | 3,366   | 122.2 MB/s |
| HOCR Invoice           |   4 KB | 20,424  |  83.6 MB/s |

The Elixir binding matches the throughput of the Rust core since conversions
are executed inside the same NIF. The numbers above help size workloads and
will be refreshed once the Elixir harness adapter lands.

## Testing

```bash
# From the repo root
task elixir:test
task elixir:lint
```
