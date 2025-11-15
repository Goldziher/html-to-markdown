defmodule HtmlToMarkdownTest do
  use ExUnit.Case, async: true

  alias HtmlToMarkdown.InlineImage

  @data_uri "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAuMBg6zidhwAAAAASUVORK5CYII="

  test "convert/1 returns markdown" do
    assert {:ok, markdown} = HtmlToMarkdown.convert("<h1>Hello</h1>")
    assert markdown =~ "# Hello"
  end

  test "convert!/2 raises on invalid options" do
    assert_raise HtmlToMarkdown.Error, fn ->
      HtmlToMarkdown.convert!("<div>", heading_style: :invalid)
    end
  end

  test "convert/2 reports invalid options" do
    assert {:error, reason} = HtmlToMarkdown.convert("<div>", heading_style: :invalid)
    assert is_binary(reason)
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

  test "options/1 raises on invalid configuration" do
    assert_raise HtmlToMarkdown.Error, fn ->
      HtmlToMarkdown.options(heading_style: :invalid)
    end
  end

  test "convert_with_options/2 uses reusable handles" do
    handle = HtmlToMarkdown.options(wrap: true, wrap_width: 5)

    assert {:ok, markdown} = HtmlToMarkdown.convert_with_options("<p>Body</p>", handle)
    assert markdown =~ "Body"
  end

  test "convert/2 rejects invalid boolean options" do
    assert {:error, reason} = HtmlToMarkdown.convert("<p>Body</p>", wrap: "yes")
    assert is_binary(reason)
    assert String.contains?(reason, "wrap")
  end

  test "convert_with_inline_images/3 extracts image payloads" do
    html = """
    <p>
      Example <img src=\"data:image/png;base64,#{@data_uri}\" alt=\"Logo\" />
    </p>
    """

    assert {:ok, markdown, images, warnings} = HtmlToMarkdown.convert_with_inline_images(html)

    assert markdown =~ "Example"
    assert warnings == []
    assert [%InlineImage{} = image] = images
    assert image.format == "png"
    assert byte_size(image.data) > 0
    assert image.source == "img_data_uri"
  end

  test "convert_with_inline_images/3 validates inline config" do
    assert {:error, reason} =
             HtmlToMarkdown.convert_with_inline_images("<p>n/a</p>", nil,
               max_decoded_size_bytes: "zero"
             )

    assert is_binary(reason)
    assert String.contains?(reason, "max_decoded_size_bytes")
  end
end
