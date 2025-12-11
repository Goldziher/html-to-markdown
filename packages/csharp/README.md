# html-to-markdown C# Bindings

High-performance HTML to Markdown converter with C# bindings to the Rust core library.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg?logo=rust&label=crates.io)](https://crates.io/crates/html-to-markdown-rs)
[![npm (node)](https://img.shields.io/npm/v/html-to-markdown-node.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://img.shields.io/npm/v/html-to-markdown-wasm.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://img.shields.io/pypi/v/html-to-markdown.svg?logo=pypi)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/Goldziher.HtmlToMarkdown.svg)](https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

## Installation

> **NuGet package ID**
>
> NuGet package names are global. To avoid clashing with an older community package named `HtmlToMarkdown`, our official bindings are published as **`Goldziher.HtmlToMarkdown`**. Use that ID in all `dotnet` CLI commands.

```bash
dotnet add package Goldziher.HtmlToMarkdown
```

## Prerequisites

The native library `html_to_markdown_ffi` must be available:

### Windows
```bash
cargo build --release -p html-to-markdown-ffi
copy target\release\html_to_markdown_ffi.dll %WINDIR%\System32\
```

### Linux
```bash
cargo build --release -p html-to-markdown-ffi
sudo cp target/release/libhtml_to_markdown_ffi.so /usr/local/lib/
sudo ldconfig
```

### macOS
```bash
cargo build --release -p html-to-markdown-ffi
sudo cp target/release/libhtml_to_markdown_ffi.dylib /usr/local/lib/
```

## Usage

```csharp
using HtmlToMarkdown;

var html = "<h1>Hello World</h1><p>This is a paragraph.</p>";

try
{
    var markdown = HtmlToMarkdownConverter.Convert(html);
    Console.WriteLine(markdown);
}
catch (HtmlToMarkdownException ex)
{
    Console.Error.WriteLine($"Conversion failed: {ex.Message}");
}
```

## API

### `HtmlToMarkdownConverter.Convert(string html)`

Converts HTML to Markdown. Throws `HtmlToMarkdownException` on error.

### `HtmlToMarkdownConverter.GetVersion()`

Returns the library version string.

### `HtmlToMarkdownConverter.ConvertWithMetadata(string html)`

Converts HTML to Markdown while extracting comprehensive metadata including document information, headers, links, images, and structured data.

Returns a `MetadataExtraction` result containing both the converted markdown and extracted metadata. Throws `HtmlToMarkdownException` on conversion or metadata parsing failure.

## Metadata Extraction

The `ConvertWithMetadata()` method extracts rich document metadata in a single pass. This is useful for SEO analysis, content management systems, web scrapers, and document processing pipelines.

### Method Signature

```csharp
/// <summary>
/// Converts HTML to Markdown and extracts comprehensive metadata.
/// </summary>
/// <param name="html">The HTML string to convert</param>
/// <returns>A MetadataExtraction result containing both markdown and extracted metadata</returns>
/// <exception cref="ArgumentNullException">Thrown when html is null</exception>
/// <exception cref="HtmlToMarkdownException">Thrown when conversion or metadata extraction fails</exception>
/// <exception cref="JsonException">Thrown when metadata JSON deserialization fails</exception>
public static MetadataExtraction ConvertWithMetadata(string html)
```

### Metadata Record Types

The metadata API uses C# records for immutable, type-safe access. All metadata is extracted into an `ExtendedMetadata` object containing:

#### `ExtendedMetadata`

Top-level metadata container:

```csharp
public record ExtendedMetadata(
    DocumentMetadata Document,
    List<HeaderMetadata> Headers,
    List<LinkMetadata> Links,
    List<ImageMetadata> Images,
    List<StructuredData> StructuredData
);
```

#### `DocumentMetadata`

Document-level metadata from `<head>` and top-level elements:

```csharp
public record DocumentMetadata(
    string? Title,                           // <title> content
    string? Description,                     // meta[name="description"]
    List<string>? Keywords,                  // meta[name="keywords"], split on commas
    string? Author,                          // meta[name="author"]
    string? CanonicalUrl,                    // <link rel="canonical">
    string? BaseHref,                        // <base href="">
    string? Language,                        // lang attribute
    string? TextDirection,                   // dir attribute (ltr, rtl, auto)
    Dictionary<string, string>? OpenGraph,   // og:* properties
    Dictionary<string, string>? TwitterCard, // twitter:* properties
    Dictionary<string, string>? MetaTags     // Other meta tags
);
```

#### `HeaderMetadata`

Header element metadata with hierarchy tracking:

```csharp
public record HeaderMetadata(
    int Level,        // Header level (1 for h1, 6 for h6)
    string Text,      // Normalized text content
    string? Id,       // HTML id attribute
    int Depth,        // Document tree depth
    int HtmlOffset    // Byte offset in original HTML
);
```

#### `LinkMetadata`

Hyperlink metadata with classification:

```csharp
public record LinkMetadata(
    string Href,                         // href attribute value
    string Text,                         // Link text content
    string? Title,                       // title attribute
    string LinkType,                     // Classification (anchor, internal, external, email, phone, other)
    List<string>? Rel,                   // rel attribute values
    Dictionary<string, string>? Attributes // Additional HTML attributes
);
```

#### `ImageMetadata`

Image metadata with source and dimensions:

```csharp
public record ImageMetadata(
    string Src,                          // Image source (URL, data URI, or SVG identifier)
    string? Alt,                         // alt attribute (accessibility)
    string? Title,                       // title attribute
    int[]? Dimensions,                   // [width, height] if available
    string ImageType,                    // Classification (data_uri, inline_svg, external, relative)
    Dictionary<string, string>? Attributes // Additional HTML attributes
);
```

#### `StructuredData`

Machine-readable structured data blocks:

```csharp
public record StructuredData(
    string DataType,   // json_ld, microdata, rdfa
    string RawJson,    // Raw JSON string (for JSON-LD)
    string? SchemaType // Schema type (e.g., "Article", "Event")
);
```

#### `MetadataExtraction`

Result container combining markdown and extracted metadata:

```csharp
public record MetadataExtraction(
    string Markdown,
    ExtendedMetadata Metadata
);
```

### Metadata Examples

#### Basic Metadata Extraction with Exception Handling

Extract and access document metadata:

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

#### Processing Headers with Hierarchy

Extract and traverse document structure:

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

#### Analyzing Links with Type Classification

Extract and filter links by type:

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

### P/Invoke Integration Notes

The `ConvertWithMetadata()` method uses P/Invoke for native FFI integration:

1. **Memory Management**: Uses `Marshal` for safe conversion between managed and unmanaged strings
2. **String Conversion**: Transparently converts between C# strings and C strings (ANSI)
3. **JSON Deserialization**: Metadata JSON is parsed to typed records using `System.Text.Json` with case-insensitive property matching
4. **Exception Handling**: All FFI errors are wrapped in `HtmlToMarkdownException` for idiomatic error handling
5. **Error Recovery**: Properly frees unmanaged memory in `finally` blocks even on exceptions

No manual pointer arithmetic is required - `Marshal` and `finally` blocks handle all cleanup.

### System.Text.Json Configuration

The metadata deserializer uses these settings for robust JSON parsing:

```csharp
var options = new JsonSerializerOptions
{
    PropertyNameCaseInsensitive = true,  // Handle snake_case from Rust
    DefaultBufferSize = 16384            // Efficient for typical metadata sizes
};
```

## Testing

```bash
cd packages/csharp
dotnet test HtmlToMarkdown.Tests/HtmlToMarkdown.Tests.csproj
```

## Performance

The Rust-backed implementation provides excellent performance:

| Document Type          | Size   | Ops/sec  | Throughput |
| ---------------------- | ------ | -------- | ---------- |
| Lists (Timeline)       | 129 KB | 1,351    | 170.6 MB/s |
| Tables (Countries)     | 360 KB | 322      | 113.3 MB/s |
| Medium (Python)        | 656 KB | 163      | 104.5 MB/s |
| Large (Rust)           | 567 KB | 180      | 99.9 MB/s  |
| Small (Intro)          | 463 KB | 184      | 83.3 MB/s  |
| HOCR German PDF        | 44 KB  | 2,667    | 113.8 MB/s |
| HOCR Invoice           | 4 KB   | 27,795   | 113.7 MB/s |
| HOCR Embedded Tables   | 37 KB  | 2,933    | 106.5 MB/s |

## Publishing to NuGet

### 1. Build the package

```bash
cd packages/csharp/HtmlToMarkdown
dotnet pack --configuration Release
```

### 2. Get NuGet API Key

1. Create account at [nuget.org](https://www.nuget.org/)
2. Go to Account → API Keys
3. Create new API key with push permissions

### 3. Publish

```bash
dotnet nuget push bin/Release/Goldziher.HtmlToMarkdown.2.8.0.nupkg \
    --api-key YOUR_API_KEY \
    --source https://api.nuget.org/v3/index.json
```

### 4. Verify

Check your package at: https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/

## License

MIT
