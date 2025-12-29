# Metadata Extraction - Elixir

Extract structured metadata from HTML documents during conversion.

## Basic Metadata Extraction

Use `convert_with_metadata/3` to extract document metadata alongside Markdown:

```elixir
html = """
<html>
  <head>
    <title>Example</title>
    <meta name="description" content="Demo page">
  </head>
  <body>
    <h1 id="welcome">Welcome</h1>
    <a href="https://example.com" rel="nofollow external">Example link</a>
  </body>
</html>
"""

{:ok, markdown, metadata} = HtmlToMarkdown.convert_with_metadata(html)

metadata["document"]["title"]        # "Example"
metadata["headers"] |> hd() |> Map.get("text") # "Welcome"
metadata["links"]   |> hd() |> Map.get("link_type") # "external"
```

## Extracted Metadata Structure

The metadata map includes:

- **Document**: Title and meta tags from `<head>`
- **Headers**: All headings extracted with level, text, and optional ID
- **Links**: All links with href, text, rel attributes, and link_type classification
- **Images**: Image sources and alt text
- **Forms**: Form action and method data
- **Other**: Tables, code blocks, and additional structural information

## Inline Image Extraction with Metadata

Combine inline image extraction with metadata:

```elixir
html = ~S(<p><img src="data:image/png;base64,..." alt="Logo"></p>)
config = %InlineImageConfig{infer_dimensions: true}

{:ok, markdown, inline_images, warnings} =
  HtmlToMarkdown.convert_with_inline_images(html, %{wrap: false}, config)

Enum.each(inline_images, fn image ->
  File.write!("output/#{image.filename}", image.data)
end)
```

## InlineImage Structure

Extracted inline images have these fields:

- **data**: Raw bytes decoded from the `<img>` or inline `<svg>`
- **format**: Subtype string (e.g., "png" or "svg")
- **filename**: Optional DOM metadata filename
- **description**: Optional DOM metadata description
- **dimensions**: `{width, height}` tuple when dimension inference is enabled
- **source**: "img_data_uri" or "svg_element" indicating where the payload originated
- **attributes**: Remaining DOM attributes preserved as a map

## InlineImageWarning Structure

Warnings from extraction include:

- **index**: Zero-based position in the inline image list
- **message**: Description of the warning

Use the index to correlate warnings back to the corresponding image in the extracted list.
