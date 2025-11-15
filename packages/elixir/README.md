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
iex> {:ok, markdown} = HtmlToMarkdown.convert("<h1>Hello</h1>")
iex> markdown
"# Hello\n"

iex> HtmlToMarkdown.convert!("<p>Example</p>", wrap: true, wrap_width: 20)
"Example\n"
```

Supported options mirror the Rust `ConversionOptions` structure. Only a subset is
currently exposed:

- `:wrap` / `:wrap_width`
- `:heading_style` (`:atx`, `:atx_closed`, `:underlined`)
- `:list_indent_type` (`:spaces`, `:tabs`)
- `:newline_style` (`:spaces`, `:backslash`)
- `:code_block_style` (`:indented`, `:backticks`, `:tildes`)
- `:whitespace` (`:normalized`, `:strict`)
- `:convert_as_inline`
- `:preprocessing` map with `:enabled`, `:preset`, `:remove_navigation`, `:remove_forms`
- `:debug`

## Testing

```
mix test
```
