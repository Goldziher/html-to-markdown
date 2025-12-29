# Conversion with Options - Elixir

Configure HTML to Markdown conversion with reusable options.

## Pre-built Options

Build reusable options using the `%HtmlToMarkdown.Options{}` struct:

```elixir
# Pre-build reusable options
iex> handle = HtmlToMarkdown.options(%Options{wrap: true, wrap_width: 40})
iex> HtmlToMarkdown.convert_with_options("<p>Reusable</p>", handle)
{:ok, "Reusable\n"}
```

## Supported Options

The `%HtmlToMarkdown.Options{}` struct mirrors the Rust `ConversionOptions` and supports:

- **Heading Style**: `heading_style` - atom values (`:atx`, etc.) mirroring Rust enums
- **List Formatting**: `list_indent_type` - control list indentation (`:tabs`, `:spaces`, etc.)
- **Output Format**: `newline_style`, `code_block_style` - atom values for various formatting styles
- **Text Wrapping**: `wrap` (boolean) and `wrap_width` (integer) - enable CommonMark soft breaks and configure column width
- **Inline Images**: `keep_inline_images_in` - map sets or lists of tag names for special handling
- **Tag Control**: `strip_tags`, `preserve_tags` - control special tag handling
- **Preprocessing**: `preprocessing` - nested `%HtmlToMarkdown.PreprocessingOptions{}` (or maps) with toggles for `:preset`, `:remove_forms`, `:remove_navigation`, etc.
- **Debug**: `debug` - enable verbose tracing from the Rust core

## Using Keyword Lists or Maps

You can also pass options as plain maps or keyword lists:

```elixir
# Using keyword list in convert! function
HtmlToMarkdown.convert!("<p>Example</p>", wrap: true, wrap_width: 20)

# Using a map
options_map = %{wrap: true, wrap_width: 40}
HtmlToMarkdown.convert_with_options("<p>Text</p>", options_map)
```
