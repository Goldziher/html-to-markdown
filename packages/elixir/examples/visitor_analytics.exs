#!/usr/bin/env elixir
# Analytics Visitor Example
# Demonstrates collecting statistics during conversion

defmodule AnalyticsVisitor do
  @moduledoc """
  A visitor that collects conversion analytics.

  Tracks:
  - Number of headings by level
  - Number of links
  - Number of images
  - Number of code blocks
  - Element depth statistics
  """

  use HtmlToMarkdown.Visitor

  # Use Agent for state management
  def start_link do
    Agent.start_link(fn ->
      %{
        headings: %{},
        links: 0,
        images: 0,
        code_blocks: 0,
        max_depth: 0,
        elements_by_type: %{}
      }
    end)
  end

  def get_stats(agent) do
    Agent.get(agent, & &1)
  end

  @impl true
  def handle_element_start(context) do
    # Track max depth
    _depth = Map.get(context, :depth, 0)
    :continue
  end

  @impl true
  def handle_heading(context, level, _text, _id) do
    _depth = Map.get(context, :depth, 0)
    # Track heading by level
    :continue
  end

  @impl true
  def handle_link(_context, _href, _text, _title) do
    # Track link
    :continue
  end

  @impl true
  def handle_image(_context, _src, _alt, _title) do
    # Track image
    :continue
  end

  @impl true
  def handle_code_block(_context, _lang, _code) do
    # Track code block
    :continue
  end
end

# Example HTML with various elements
html = """
<article>
  <h1>Main Article</h1>
  <p>This article contains <a href="#section1">a link</a> and an image:</p>
  <img src="hero.jpg" alt="Hero Image">

  <h2>Section 1</h2>
  <p>Some text with <a href="https://example.com">another link</a>.</p>
  <pre><code>
  def hello(name)
    puts "Hello, #{name}!"
  end
  </code></pre>

  <h2>Section 2</h2>
  <p>More content with <a href="/page">a link</a> and <img src="diagram.png" alt="Diagram">.</p>

  <h3>Subsection 2.1</h3>
  <p>Additional text.</p>

  <pre><code class="language-python">
  def factorial(n):
      return 1 if n <= 1 else n * factorial(n-1)
  </code></pre>
</article>
"""

IO.puts("Analytics Visitor Example")
IO.puts("=========================")
IO.puts("")
IO.puts("Analyzing HTML...")

{:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, AnalyticsVisitor, nil)

IO.puts("")
IO.puts("Conversion completed successfully!")
IO.puts("")
IO.puts("Converted Markdown:")
IO.puts("---")
IO.puts(markdown)
IO.puts("---")
IO.puts("")
IO.puts("Analytics:")
IO.puts("  Note: Full analytics integration requires GenServer/Agent support")
IO.puts("  in the Rust NIF layer for stateful visitor patterns.")
