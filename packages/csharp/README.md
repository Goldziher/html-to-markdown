# html-to-markdown C# Bindings

High-performance HTML to Markdown converter with C# bindings to the Rust core library.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg?logo=rust&label=crates.io)](https://crates.io/crates/html-to-markdown-rs)
[![npm (node)](https://img.shields.io/npm/v/%40kreuzberg%2Fhtml-to-markdown-node.svg?logo=npm)](https://www.npmjs.com/package/@kreuzberg/html-to-markdown-node)
[![npm (wasm)](https://img.shields.io/npm/v/%40kreuzberg%2Fhtml-to-markdown-wasm.svg?logo=npm)](https://www.npmjs.com/package/@kreuzberg/html-to-markdown-wasm)
[![PyPI](https://img.shields.io/pypi/v/html-to-markdown.svg?logo=pypi)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/KreuzbergDev.HtmlToMarkdown.svg)](https://www.nuget.org/packages/KreuzbergDev.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/dev.kreuzberg/html-to-markdown.svg)](https://central.sonatype.com/artifact/dev.kreuzberg/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown.svg)](https://pkg.go.dev/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

## Installation

> **NuGet package ID**
>
> NuGet package names are global. To avoid clashing with an older community package named `HtmlToMarkdown`, our official bindings are published as **`KreuzbergDev.HtmlToMarkdown`**. Use that ID in all `dotnet` CLI commands.

```bash
dotnet add package KreuzbergDev.HtmlToMarkdown
```

## Migration Guide (v2.18.x → v2.19.0)

### Breaking Change: Package Owner Update

In v2.19.0, the C#/.NET package owner changed from `Goldziher` to `KreuzbergDev` to reflect the new Kreuzberg.dev organization.

#### NuGet Update

**Before (v2.18.x):**
```bash
dotnet add package Goldziher.HtmlToMarkdown
```

**After (v2.19.0+):**
```bash
dotnet add package KreuzbergDev.HtmlToMarkdown
```

#### Project File Update

Update your `.csproj` file:

**Before:**
```xml
<PackageReference Include="Goldziher.HtmlToMarkdown" Version="2.18.x" />
```

**After:**
```xml
<PackageReference Include="KreuzbergDev.HtmlToMarkdown" Version="2.19.0" />
```

#### Using Statement Update

**Before (may also work):**
```csharp
using HtmlToMarkdown;
```

**After (recommended):**
```csharp
using HtmlToMarkdown;  // Same namespace, package owner changed
```

#### Summary of Changes

- Package renamed from `Goldziher.HtmlToMarkdown` to `KreuzbergDev.HtmlToMarkdown`
- Using statements remain the same
- All APIs are identical
- No code changes required beyond updating the package reference

## Prerequisites

The NuGet package ships the managed bindings; the native library is provided via Rust FFI (`html_to_markdown_ffi`).

If you're running from source (or on an unsupported platform), ensure the native library is available:

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

## Visitor Pattern

The visitor pattern provides fine-grained control over HTML element processing during conversion. Implement the `IVisitor` interface to intercept visitor callbacks for specific element types, modify conversion behavior, track metadata, or filter content.

### Basic Usage

```csharp
using HtmlToMarkdown;
using HtmlToMarkdown.Visitor;

// Create a custom visitor
class MyVisitor : IVisitor
{
    public VisitResult VisitHeading(NodeContext context, int level, string text, string? id)
    {
        Console.WriteLine($"H{level}: {text}");
        return VisitResult.Continue();  // Use default conversion
    }

    public VisitResult VisitLink(NodeContext context, string href, string text, string? title)
    {
        // Example: replace links to a specific domain
        if (href.Contains("blocked-site.com"))
            return VisitResult.Skip();  // Omit this link entirely

        return VisitResult.Continue();
    }
}

var html = "<h1>Welcome</h1><p>See <a href=\"https://example.com\">example</a></p>";
var visitor = new MyVisitor();
var markdown = HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);
```

### IVisitor Interface

The `IVisitor` interface includes 40+ callback methods for customizing element processing:

#### Generic Hooks
- `VisitResult VisitElementStart(NodeContext context)` — Before entering any HTML element
- `VisitResult VisitElementEnd(NodeContext context, string output)` — After exiting any element with default output

#### Text and Content
- `VisitResult VisitText(NodeContext context, string text)` — Text nodes (most frequent: 100+ per document)
- `VisitResult VisitCodeBlock(NodeContext context, string? lang, string code)` — Code blocks `<pre><code>`
- `VisitResult VisitCodeInline(NodeContext context, string code)` — Inline code `<code>`

#### Document Structure
- `VisitResult VisitHeading(NodeContext context, int level, string text, string? id)` — Headings `<h1>-<h6>`
- `VisitResult VisitBlockquote(NodeContext context, string content, int depth)` — Blockquotes `<blockquote>`

#### Links and Images
- `VisitResult VisitLink(NodeContext context, string href, string text, string? title)` — Anchor links `<a>`
- `VisitResult VisitImage(NodeContext context, string src, string alt, string? title)` — Images `<img>`

#### Lists
- `VisitResult VisitListStart(NodeContext context, bool ordered)` — Before list `<ul>/<ol>`
- `VisitResult VisitListItem(NodeContext context, bool ordered, string marker, string text)` — List items `<li>`
- `VisitResult VisitListEnd(NodeContext context, bool ordered, string output)` — After list

#### Tables
- `VisitResult VisitTableStart(NodeContext context)` — Before table `<table>`
- `VisitResult VisitTableRow(NodeContext context, IReadOnlyList<string> cells, bool isHeader)` — Table rows `<tr>`
- `VisitResult VisitTableEnd(NodeContext context, string output)` — After table

#### Inline Formatting
- `VisitResult VisitStrong(NodeContext context, string text)` — Bold `<strong>/<b>`
- `VisitResult VisitEmphasis(NodeContext context, string text)` — Italic `<em>/<i>`
- `VisitResult VisitStrikethrough(NodeContext context, string text)` — Strikethrough `<s>/<del>`
- `VisitResult VisitUnderline(NodeContext context, string text)` — Underline `<u>/<ins>`
- `VisitResult VisitSubscript(NodeContext context, string text)` — Subscript `<sub>`
- `VisitResult VisitSuperscript(NodeContext context, string text)` — Superscript `<sup>`
- `VisitResult VisitMark(NodeContext context, string text)` — Highlight `<mark>`

#### Breaks and Separators
- `VisitResult VisitLineBreak(NodeContext context)` — Line breaks `<br>`
- `VisitResult VisitHorizontalRule(NodeContext context)` — Horizontal rules `<hr>`

#### Forms
- `VisitResult VisitForm(NodeContext context, string? action, string? method)` — Form elements `<form>`
- `VisitResult VisitInput(NodeContext context, string inputType, string? name, string? value)` — Input fields `<input>`
- `VisitResult VisitButton(NodeContext context, string text)` — Buttons `<button>`

#### Media
- `VisitResult VisitAudio(NodeContext context, string? src)` — Audio elements `<audio>`
- `VisitResult VisitVideo(NodeContext context, string? src)` — Video elements `<video>`
- `VisitResult VisitIFrame(NodeContext context, string? src)` — Embedded frames `<iframe>`

#### Semantic HTML5
- `VisitResult VisitDetails(NodeContext context, bool open)` — Disclosure triangles `<details>`
- `VisitResult VisitSummary(NodeContext context, string text)` — Summary text `<summary>`
- `VisitResult VisitFigureStart(NodeContext context)` — Figures `<figure>`
- `VisitResult VisitFigCaption(NodeContext context, string text)` — Figure captions `<figcaption>`

#### Definition Lists
- `VisitResult VisitDefinitionListStart(NodeContext context)` — Definition lists `<dl>`
- `VisitResult VisitDefinitionTerm(NodeContext context, string text)` — Terms `<dt>`
- `VisitResult VisitDefinitionDescription(NodeContext context, string text)` — Descriptions `<dd>`

#### Custom Elements
- `VisitResult VisitCustomElement(NodeContext context, string tagName, string html)` — Web components and unknown tags

All callback methods have default implementations that return `VisitResult.Continue()`. Override only the callbacks you need.

### VisitResult Types

Control how the converter proceeds after a visitor callback:

```csharp
// Continue with default conversion behavior
return VisitResult.Continue();

// Replace element output with custom markdown
return VisitResult.Custom("**custom markdown**");

// Skip element and all children entirely
return VisitResult.Skip();

// Preserve original HTML instead of converting
return VisitResult.PreserveHtml();

// Stop conversion and report error
return VisitResult.Error("Conversion error: " + ex.Message);
```

### NodeContext Properties

Visitor callbacks receive a `NodeContext` providing metadata about the current element:

```csharp
public class NodeContext
{
    // Element classification
    public NodeType NodeType { get; }
    public string TagName { get; }

    // Attributes
    public IReadOnlyList<Attribute> Attributes { get; }
    public string? GetAttribute(string key);
    public bool HasAttribute(string key);

    // DOM position
    public int Depth { get; }
    public int IndexInParent { get; }
    public string? ParentTag { get; }

    // Inline vs block
    public bool IsInline { get; }
}
```

### Examples

#### Example 1: Track Document Structure

```csharp
class StructureVisitor : IVisitor
{
    public List<(int Level, string Text)> Headings { get; } = new();
    public List<(string Href, string Text)> Links { get; } = new();

    public VisitResult VisitHeading(NodeContext context, int level, string text, string? id)
    {
        Headings.Add((level, text));
        return VisitResult.Continue();
    }

    public VisitResult VisitLink(NodeContext context, string href, string text, string? title)
    {
        Links.Add((href, text));
        return VisitResult.Continue();
    }
}

var visitor = new StructureVisitor();
HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);

foreach (var (level, text) in visitor.Headings)
{
    Console.WriteLine($"H{level}: {text}");
}
```

#### Example 2: Filter Content

```csharp
class FilterVisitor : IVisitor
{
    public VisitResult VisitElementStart(NodeContext context)
    {
        // Skip all advertisement divs
        if (context.TagName == "div" && context.GetAttribute("class")?.Contains("ad") == true)
            return VisitResult.Skip();

        // Skip script and style tags
        if (context.TagName is "script" or "style")
            return VisitResult.Skip();

        return VisitResult.Continue();
    }

    public VisitResult VisitImage(NodeContext context, string src, string alt, string? title)
    {
        // Skip tracking pixels and social share buttons
        if (src.Contains("pixel") || src.Contains("tracking"))
            return VisitResult.Skip();

        return VisitResult.Continue();
    }
}
```

#### Example 3: Customize Output

```csharp
class CustomizingVisitor : IVisitor
{
    public VisitResult VisitLink(NodeContext context, string href, string text, string? title)
    {
        // Convert external links to reference-style markdown
        var uri = new Uri(href, UriKind.RelativeOrAbsolute);
        if (uri.IsAbsoluteUri && uri.Host != "example.com")
        {
            // Return custom markdown with footnote
            return VisitResult.Custom($"[{text}][^external-{href.GetHashCode()}]");
        }

        return VisitResult.Continue();
    }

    public VisitResult VisitHeading(NodeContext context, int level, string text, string? id)
    {
        // Add custom attributes to headings
        if (string.IsNullOrEmpty(id))
        {
            var slugId = text.ToLower().Replace(" ", "-");
            return VisitResult.Custom($"{'#'}{level} {text} {{#{slugId}}}");
        }

        return VisitResult.Continue();
    }
}
```

### Performance Notes

- Text callbacks (`VisitText`) are called 100+ times per document — keep them fast
- Visitor state is isolated per conversion (thread-safe by design)
- Delegates are marshalled once at visitor creation time (no per-callback overhead)
- All string conversions from C FFI happen automatically (UTF-8 safe)

### Error Handling

Exceptions in visitor callbacks are caught and converted to `VisitResult.Error()`. The error message is propagated to the caller:

```csharp
class SafeVisitor : IVisitor
{
    public VisitResult VisitLink(NodeContext context, string href, string text, string? title)
    {
        try
        {
            var uri = new Uri(href);  // May throw for invalid URIs
            return VisitResult.Continue();
        }
        catch (UriFormatException ex)
        {
            return VisitResult.Error($"Invalid link URI: {ex.Message}");
        }
    }
}
```

## Performance

The Rust-backed implementation provides excellent performance:

| Document Type          | Size   | Ops/sec  | Throughput |
| ---------------------- | ------ | -------- | ---------- |
| Lists (Timeline)       | 129 KB | 3,111    | 392.9 MB/s |
| Tables (Countries)     | 360 KB | 853      | 300.1 MB/s |
| Medium (Python)        | 656 KB | 456      | 292.3 MB/s |
| Large (Rust)           | 567 KB | 533      | 295.2 MB/s |
| Small (Intro)          | 463 KB | 571      | 258.5 MB/s |
| HOCR German PDF        | 44 KB  | 6,534    | 278.7 MB/s |
| HOCR Invoice           | 4 KB   | 84,529   | 345.9 MB/s |
| HOCR Embedded Tables   | 37 KB  | 7,856    | 285.1 MB/s |

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
dotnet nuget push bin/Release/KreuzbergDev.HtmlToMarkdown.2.18.0.nupkg \
    --api-key YOUR_API_KEY \
    --source https://api.nuget.org/v3/index.json
```

### 4. Verify

Check your package at: https://www.nuget.org/packages/KreuzbergDev.HtmlToMarkdown/

## License

MIT
