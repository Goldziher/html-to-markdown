defmodule HtmlToMarkdownTestApp.SmokeTest do
  use ExUnit.Case

  test "package loads" do
    assert Code.ensure_loaded?(HtmlToMarkdown)
  end

  test "basic conversion" do
    html = "<p>Hello World</p>"
    result = HtmlToMarkdown.convert(html)
    assert String.contains?(result, "Hello World")
  end

  test "with options" do
    html = "<h1>Title</h1>"
    result = HtmlToMarkdown.convert(html)
    assert String.starts_with?(result, "#")
  end

  test "empty input" do
    result = HtmlToMarkdown.convert("")
    assert result == ""
  end
end
