# Visitor Pattern - Elixir

Customize HTML to Markdown conversion by implementing visitor callbacks.

## Basic Visitor Example

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

## Available Callbacks

### Generic Hooks

- `handle_element_start(context)` - called before entering any element
- `handle_element_end(context, output)` - called after exiting an element

### Text & Formatting

- `handle_text(context, text)` - text nodes
- `handle_strong(context, text)` - `<strong>`, `<b>`
- `handle_emphasis(context, text)` - `<em>`, `<i>`
- `handle_strikethrough(context, text)` - `<s>`, `<del>`, `<strike>`
- `handle_underline(context, text)` - `<u>`, `<ins>`
- `handle_subscript(context, text)` - `<sub>`
- `handle_superscript(context, text)` - `<sup>`
- `handle_mark(context, text)` - `<mark>`

### Links & Media

- `handle_link(context, href, text, title)` - `<a>` elements
- `handle_image(context, src, alt, title)` - `<img>` elements
- `handle_audio(context, src)` - `<audio>` elements
- `handle_video(context, src)` - `<video>` elements
- `handle_iframe(context, src)` - `<iframe>` elements

### Code

- `handle_code_block(context, lang, code)` - `<pre><code>` blocks
- `handle_code_inline(context, code)` - `<code>` inline

### Headings & Structure

- `handle_heading(context, level, text, id)` - `<h1>` through `<h6>`
- `handle_blockquote(context, content, depth)` - `<blockquote>`
- `handle_horizontal_rule(context)` - `<hr>`
- `handle_line_break(context)` - `<br>`

### Lists

- `handle_list_start(context, ordered)` - `<ul>` or `<ol>` start
- `handle_list_item(context, ordered, marker, text)` - `<li>` elements
- `handle_list_end(context, ordered, output)` - list end

### Tables

- `handle_table_start(context)` - `<table>` start
- `handle_table_row(context, cells, is_header)` - `<tr>` elements
- `handle_table_end(context, output)` - table end

### Forms

- `handle_form(context, action, method)` - `<form>`
- `handle_input(context, type, name, value)` - `<input>`
- `handle_button(context, text)` - `<button>`

### Definition Lists

- `handle_definition_list_start(context)` - `<dl>` start
- `handle_definition_term(context, text)` - `<dt>`
- `handle_definition_description(context, text)` - `<dd>`
- `handle_definition_list_end(context, output)` - list end

### Custom Elements

- `handle_custom_element(context, tag_name, html)` - web components or unknown tags
- `handle_other(callback, context, args)` - catch-all for unimplemented callbacks

## Visitor Return Values

Each callback must return one of:

- `:continue` - proceed with default conversion
- `{:custom, markdown}` - replace output with custom markdown
- `:skip` - omit this element entirely
- `:preserve_html` - include raw HTML verbatim
- `{:error, reason}` - stop conversion with error

## Node Context

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

## Remove All Links Example

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

## Advanced Example: Stateful Image Collection

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

## Execution Order

Callbacks are invoked during depth-first traversal. For `<div><p>text</p></div>`:

1. `handle_element_start` for `<div>`
2. `handle_element_start` for `<p>`
3. `handle_text` for "text"
4. `handle_element_end` for `<p>`
5. `handle_element_end` for `</div>`
