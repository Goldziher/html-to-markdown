# HtmlToMarkdown (Elixir)

Elixir bindings for the Rust [html-to-markdown](https://github.com/Goldziher/html-to-markdown) engine.
The package exposes a fast `HTML -> Markdown` converter implemented with Rustler.

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

## Testing

```bash
# From the repo root
task elixir:test
task elixir:lint
```
