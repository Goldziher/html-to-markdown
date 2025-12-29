# Basic Usage - Elixir

Basic HTML to Markdown conversion using HtmlToMarkdown.

## Simple Conversion

Convert HTML to Markdown with default options:

```elixir
iex> {:ok, markdown} = HtmlToMarkdown.convert("<h1>Hello</h1>")
iex> markdown
"# Hello\n"
```

## Conversion with Bang Operator

Use the bang version for inline conversions with immediate options:

```elixir
iex> HtmlToMarkdown.convert!("<p>Example</p>", wrap: true, wrap_width: 20)
"Example\n"
```

## Import Aliases

For cleaner code, import the necessary modules:

```elixir
alias HtmlToMarkdown.{InlineImageConfig, Options}
```
