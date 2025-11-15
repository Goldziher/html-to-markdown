defmodule HtmlToMarkdownTest do
  use ExUnit.Case, async: true

  test "convert/1 returns markdown" do
    assert {:ok, markdown} = HtmlToMarkdown.convert("<h1>Hello</h1>")
    assert markdown =~ "# Hello"
  end

  test "convert!/2 raises on invalid options" do
    assert_raise HtmlToMarkdown.Error, fn ->
      HtmlToMarkdown.convert!("<div>", heading_style: :invalid)
    end
  end

  test "convert/2 accepts keyword options" do
    assert {:ok, markdown} =
             HtmlToMarkdown.convert("<p>Example</p>",
               wrap: true,
               wrap_width: 10,
               preprocessing: %{enabled: true, preset: :minimal}
             )

    assert markdown =~ "Example"
  end
end
