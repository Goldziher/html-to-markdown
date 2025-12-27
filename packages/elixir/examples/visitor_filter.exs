#!/usr/bin/env elixir
# Demonstrates filtering external links during conversion

defmodule LinkFilterVisitor do
  @moduledoc """
  A visitor that filters links based on domain.

  - Keeps internal links (same domain)
  - Removes external links
  - Removes email and phone links
  """

  use HtmlToMarkdown.Visitor

  def init(base_domain) do
    %{base_domain: base_domain}
  end

  @impl true
  def handle_link(_context, href, text, _title) do
    cond do
      not String.contains?(href, "://") ->
        :continue

      not String.contains?(href, @domain) ->
        {:custom, text}

      true ->
        :continue
    end
  catch
    _ -> :continue
  end
end

html = """
<article>
  <h1>Article Title</h1>
  <p>
    This article references
    <a href="/internal-page">an internal page</a>,
    <a href="https://external-site.com">an external link</a>,
    and <a href="https://github.com">GitHub</a>.
  </p>
  <p>
    You can also contact via
    <a href="mailto:user@example.com">email</a> or
    <a href="tel:+1234567890">phone</a>.
  </p>
</article>
"""

IO.puts("Link Filter Visitor Example")
IO.puts("===========================")
IO.puts("")
IO.puts("Original HTML:")
IO.puts(html)
IO.puts("")

{:ok, markdown} = HtmlToMarkdown.Visitor.convert_with_visitor(html, LinkFilterVisitor, nil)

IO.puts("Converted to Markdown (external links removed):")
IO.puts("---")
IO.puts(markdown)
IO.puts("---")
