# C# Metadata Extraction

Extract rich document metadata while converting HTML to Markdown.

## Basic Metadata Extraction

```csharp
using HtmlToMarkdown;

try
{
    string html = """
        <html>
        <head>
            <title>My Article</title>
            <meta name="description" content="An interesting read">
            <meta name="author" content="Jane Doe">
            <meta property="og:image" content="image.jpg">
        </head>
        <body>
            <h1>Welcome</h1>
            <a href="https://example.com">Link</a>
            <img src="image.jpg" alt="Featured image">
        </body>
        </html>
        """;

    var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

    // Access document metadata
    var doc = result.Metadata.Document;
    if (doc.Title != null)
    {
        Console.WriteLine($"Title: {doc.Title}");
    }
    if (doc.Author != null)
    {
        Console.WriteLine($"Author: {doc.Author}");
    }

    // Access Open Graph metadata
    if (doc.OpenGraph != null)
    {
        foreach (var (key, value) in doc.OpenGraph)
        {
            Console.WriteLine($"OG {key}: {value}");
        }
    }

    // Count extracted elements
    Console.WriteLine($"Headers: {result.Metadata.Headers.Count}");
    Console.WriteLine($"Links: {result.Metadata.Links.Count}");
    Console.WriteLine($"Images: {result.Metadata.Images.Count}");

    // Print markdown output
    Console.WriteLine($"\nMarkdown:\n{result.Markdown}");
}
catch (HtmlToMarkdownException ex)
{
    Console.Error.WriteLine($"Conversion failed: {ex.Message}");
}
catch (JsonException ex)
{
    Console.Error.WriteLine($"Metadata parsing failed: {ex.Message}");
}
```

## Processing Headers with Hierarchy

```csharp
using HtmlToMarkdown;

try
{
    string html = """
        <html><body>
            <h1>Main Title</h1>
            <h2>Section One</h2>
            <h3>Subsection</h3>
            <h2>Section Two</h2>
        </body></html>
        """;

    var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

    Console.WriteLine("Document Structure:");
    foreach (var header in result.Metadata.Headers)
    {
        string indent = new string(' ', (header.Level - 1) * 2);
        Console.Write($"{indent}- Level {header.Level}: {header.Text}");
        if (header.Id != null)
        {
            Console.Write($" (#{header.Id})");
        }
        Console.WriteLine();
    }
}
catch (HtmlToMarkdownException ex)
{
    Console.Error.WriteLine($"Failed: {ex.Message}");
}
```

## Analyzing Links with Type Classification

```csharp
using HtmlToMarkdown;
using System.Linq;

try
{
    string html = """
        <html><body>
            <a href="https://external.com">External Site</a>
            <a href="/internal">Internal Page</a>
            <a href="#section">Anchor Link</a>
            <a href="mailto:test@example.com">Email</a>
            <a href="tel:+1234567890">Phone</a>
            <a href="https://external.com" rel="nofollow">Nofollow Link</a>
        </body></html>
        """;

    var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

    // Group links by type
    var linksByType = result.Metadata.Links
        .GroupBy(l => l.LinkType)
        .ToDictionary(g => g.Key, g => g.ToList());

    // Process external links
    if (linksByType.TryGetValue("external", out var external) && external.Count > 0)
    {
        Console.WriteLine($"External Links: {external.Count}");
        foreach (var link in external)
        {
            Console.WriteLine($"  - {link.Text}: {link.Href}");
        }
    }

    // Find nofollow links
    var nofollow = result.Metadata.Links
        .Where(l => l.Rel?.Contains("nofollow") == true)
        .ToList();

    if (nofollow.Count > 0)
    {
        Console.WriteLine($"\nNofollow Links: {nofollow.Count}");
        foreach (var link in nofollow)
        {
            Console.WriteLine($"  - {link.Text}: {link.Href}");
        }
    }
}
catch (HtmlToMarkdownException ex)
{
    Console.Error.WriteLine($"Failed: {ex.Message}");
}
```

## API Reference

### `HtmlToMarkdownConverter.ConvertWithMetadata(string html)`

Converts HTML to Markdown while extracting comprehensive metadata including document information, headers, links, images, and structured data.

**Returns:** A `MetadataExtraction` result containing both the converted markdown and extracted metadata.

**Throws:** `HtmlToMarkdownException` on conversion or metadata parsing failure.

### Metadata Types

**`MetadataExtraction`** - Result container combining markdown and extracted metadata:
- `string Markdown` - Converted markdown output
- `ExtendedMetadata Metadata` - Extracted metadata container

**`ExtendedMetadata`** - Top-level metadata container:
- `DocumentMetadata Document` - Document-level metadata
- `List<HeaderMetadata> Headers` - Extracted headers with hierarchy
- `List<LinkMetadata> Links` - Extracted links with classification
- `List<ImageMetadata> Images` - Extracted images with metadata
- `List<StructuredData> StructuredData` - Machine-readable structured data

**`DocumentMetadata`** - Document-level metadata:
- `string? Title` - `<title>` content
- `string? Description` - meta[name="description"]
- `List<string>? Keywords` - meta[name="keywords"], split on commas
- `string? Author` - meta[name="author"]
- `string? CanonicalUrl` - `<link rel="canonical">`
- `Dictionary<string, string>? OpenGraph` - og:* properties
- `Dictionary<string, string>? TwitterCard` - twitter:* properties

**`HeaderMetadata`** - Header element metadata:
- `int Level` - Header level (1 for h1, 6 for h6)
- `string Text` - Normalized text content
- `string? Id` - HTML id attribute
- `int Depth` - Document tree depth

**`LinkMetadata`** - Hyperlink metadata:
- `string Href` - href attribute value
- `string Text` - Link text content
- `string? Title` - title attribute
- `string LinkType` - Classification (anchor, internal, external, email, phone, other)
- `List<string>? Rel` - rel attribute values

**`ImageMetadata`** - Image metadata:
- `string Src` - Image source (URL, data URI, or SVG identifier)
- `string? Alt` - alt attribute (accessibility)
- `string? Title` - title attribute
- `int[]? Dimensions` - [width, height] if available
- `string ImageType` - Classification (data_uri, inline_svg, external, relative)
