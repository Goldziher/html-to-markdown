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

### Visitor Pattern

The visitor pattern allows you to intervene in the conversion process and customize
behavior for specific HTML elements. This is useful for filtering content, collecting
metadata, applying custom formatting, or implementing content policies.

#### Basic Example

Define a visitor module implementing `HtmlToMarkdown.Visitor`:

```elixir
defmodule MyLinkFilter do
  use HtmlToMarkdown.Visitor

  @impl true
  def handle_link(_context, _href, text, _title) do
    # Convert all links to plain text
    {:custom, text}
  end
end

html = "<p>Visit <a href='https://example.com'>our site</a> for more!</p>"
{:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, MyLinkFilter, nil)
# markdown == "Visit our site for more!\n"
```

#### Available Callbacks

The visitor pattern supports callbacks for all HTML element types:

**Generic Hooks:**
- `handle_element_start(context)` - called before entering any element
- `handle_element_end(context, output)` - called after exiting an element

**Text & Formatting:**
- `handle_text(context, text)` - text nodes
- `handle_strong(context, text)` - `<strong>`, `<b>`
- `handle_emphasis(context, text)` - `<em>`, `<i>`
- `handle_strikethrough(context, text)` - `<s>`, `<del>`, `<strike>`
- `handle_underline(context, text)` - `<u>`, `<ins>`
- `handle_subscript(context, text)` - `<sub>`
- `handle_superscript(context, text)` - `<sup>`
- `handle_mark(context, text)` - `<mark>`

**Links & Media:**
- `handle_link(context, href, text, title)` - `<a>` elements
- `handle_image(context, src, alt, title)` - `<img>` elements
- `handle_audio(context, src)` - `<audio>` elements
- `handle_video(context, src)` - `<video>` elements
- `handle_iframe(context, src)` - `<iframe>` elements

**Code:**
- `handle_code_block(context, lang, code)` - `<pre><code>` blocks
- `handle_code_inline(context, code)` - `<code>` inline

**Headings & Structure:**
- `handle_heading(context, level, text, id)` - `<h1>` through `<h6>`
- `handle_blockquote(context, content, depth)` - `<blockquote>`
- `handle_horizontal_rule(context)` - `<hr>`
- `handle_line_break(context)` - `<br>`

**Lists:**
- `handle_list_start(context, ordered)` - `<ul>` or `<ol>` start
- `handle_list_item(context, ordered, marker, text)` - `<li>` elements
- `handle_list_end(context, ordered, output)` - list end

**Tables:**
- `handle_table_start(context)` - `<table>` start
- `handle_table_row(context, cells, is_header)` - `<tr>` elements
- `handle_table_end(context, output)` - table end

**Forms:**
- `handle_form(context, action, method)` - `<form>`
- `handle_input(context, type, name, value)` - `<input>`
- `handle_button(context, text)` - `<button>`

**Definition Lists:**
- `handle_definition_list_start(context)` - `<dl>` start
- `handle_definition_term(context, text)` - `<dt>`
- `handle_definition_description(context, text)` - `<dd>`
- `handle_definition_list_end(context, output)` - list end

**Custom Elements:**
- `handle_custom_element(context, tag_name, html)` - web components or unknown tags
- `handle_other(callback, context, args)` - catch-all for unimplemented callbacks

#### Visit Results

Each callback must return one of:

- `:continue` - proceed with default conversion
- `{:custom, markdown}` - replace output with custom markdown
- `:skip` - omit this element entirely
- `:preserve_html` - include raw HTML verbatim
- `{:error, reason}` - stop conversion with error

#### Node Context

All callbacks receive a `NodeContext` struct with element metadata:

```elixir
%{
  node_type: :link,           # coarse-grained classification
  tag_name: "a",              # raw HTML tag name
  attributes: %{...},         # HTML attributes as a map
  depth: 2,                   # nesting depth in DOM
  index_in_parent: 0,         # zero-based sibling index
  parent_tag: "p",            # parent element's tag (nil if root)
  is_inline: true             # whether treated as inline vs block
}
```

#### Advanced Example: Image Collection

Use a GenServer to maintain state across callbacks:

```elixir
defmodule ImageCollector do
  use GenServer
  use HtmlToMarkdown.Visitor

  def start_link(_), do: GenServer.start_link(__MODULE__, [])

  def init(_), do: {:ok, []}

  @impl true
  def handle_image(_context, src, alt, _title) do
    GenServer.cast(self(), {:collect, src, alt})
    :continue
  end

  def handle_cast({:collect, src, alt}, images) do
    {:noreply, [%{src: src, alt: alt} | images]}
  end
end

{:ok, pid} = ImageCollector.start_link(nil)
{:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, pid, nil)
# Can query collected images via GenServer API
```

#### Filtering Example: Remove All Links

```elixir
defmodule NoLinksVisitor do
  use HtmlToMarkdown.Visitor

  @impl true
  def handle_link(_context, _href, text, _title) do
    # Convert links to plain text
    {:custom, text}
  end
end

html = "<p>Check <a href='#'>this</a> out.</p>"
{:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, NoLinksVisitor, nil)
# markdown == "Check this out.\n"
```

#### Execution Order

Callbacks are invoked during depth-first traversal. For `<div><p>text</p></div>`:

1. `handle_element_start` for `<div>`
2. `handle_element_start` for `<p>`
3. `handle_text` for "text"
4. `handle_element_end` for `<p>`
5. `handle_element_end` for `</div>`

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
| Lists (Timeline)       | 129 KB | 2,547   | 321.7 MB/s |
| Tables (Countries)     | 360 KB |   835   | 293.8 MB/s |
| Medium (Python)        | 656 KB |   439   | 281.5 MB/s |
| Large (Rust)           | 567 KB |   485   | 268.7 MB/s |
| Small (Intro)          | 463 KB |   581   | 262.9 MB/s |
| HOCR German PDF        |  44 KB | 7,106   | 303.1 MB/s |
| HOCR Embedded Tables   |  37 KB | 6,231   | 226.1 MB/s |
| HOCR Invoice           |   4 KB | 62,657  | 256.4 MB/s |

The Elixir binding matches the throughput of the Rust core since conversions
are executed inside the same NIF. The numbers above help size workloads and
will be refreshed once the Elixir harness adapter lands.

## Testing

```bash
# From the repo root
task elixir:test
task elixir:lint
```
