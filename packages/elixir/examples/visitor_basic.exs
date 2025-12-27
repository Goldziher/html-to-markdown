#!/usr/bin/env elixir
# Basic Visitor Pattern Example
# Demonstrates core visitor functionality

defmodule BasicVisitor do
  @moduledoc """
  A simple visitor that tracks which callbacks are invoked.
  """

  use HtmlToMarkdown.Visitor

  @impl true
  def handle_heading(_context, level, text, _id) do
    IO.puts("  Found heading level #{level}: #{text}")
    :continue
  end

  @impl true
  def handle_link(_context, href, text, _title) do
    IO.puts("  Found link: #{text} -> #{href}")
    :continue
  end

  @impl true
  def handle_image(_context, src, alt, _title) do
    IO.puts("  Found image: #{alt} (#{src})")
    :continue
  end
end

# Example HTML
html = """
<html>
<head><title>Example</title></head>
<body>
  <h1>Welcome</h1>
  <p>This is a <a href="https://example.com">link</a> and an <img src="test.jpg" alt="Test Image"></p>
  <h2>Section</h2>
  <p>More content with <a href="https://test.com">another link</a>.</p>
</body>
</html>
"""

IO.puts("Converting HTML with BasicVisitor:")
IO.puts("")

{:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, BasicVisitor, nil)

IO.puts("")
IO.puts("Converted Markdown:")
IO.puts("---")
IO.puts(markdown)
IO.puts("---")
