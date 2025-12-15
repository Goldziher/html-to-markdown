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

  test "convert_with_metadata/3 extracts document + headers + links + images" do
    html = """
    <html>
      <head>
        <title>Example Article</title>
        <meta name="description" content="Demo page">
        <link rel="canonical" href="https://example.com/article">
      </head>
      <body>
        <h1 id="welcome">Welcome</h1>
        <a href="https://example.com" rel="nofollow external">Example link</a>
        <img src="https://example.com/image.jpg" alt="Hero" width="640" height="480">
      </body>
    </html>
    """

    assert {:ok, _markdown, metadata} = HtmlToMarkdown.convert_with_metadata(html)
    assert metadata["document"]["title"] == "Example Article"
    assert metadata["document"]["description"] == "Demo page"

    assert [%{"level" => 1, "text" => "Welcome"} | _] = metadata["headers"]
    assert [%{"href" => "https://example.com", "link_type" => "external"} | _] = metadata["links"]

    assert [%{"src" => "https://example.com/image.jpg", "image_type" => "external"} | _] =
             metadata["images"]
  end
end
