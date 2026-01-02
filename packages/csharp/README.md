# html-to-markdown

<div align="center" style="display: flex; flex-wrap: wrap; gap: 8px; justify-content: center; margin: 20px 0;">
  <!-- Language Bindings -->
  <a href="https://crates.io/crates/html-to-markdown-rs">
    <img src="https://img.shields.io/crates/v/html-to-markdown-rs?label=Rust&color=007ec6" alt="Rust">
  </a>
  <a href="https://pypi.org/project/html-to-markdown/">
    <img src="https://img.shields.io/pypi/v/html-to-markdown?label=Python&color=007ec6" alt="Python">
  </a>
  <a href="https://www.npmjs.com/package/@kreuzberg/html-to-markdown-node">
    <img src="https://img.shields.io/npm/v/@kreuzberg/html-to-markdown-node?label=Node.js&color=007ec6" alt="Node.js">
  </a>
  <a href="https://www.npmjs.com/package/@kreuzberg/html-to-markdown-wasm">
    <img src="https://img.shields.io/npm/v/@kreuzberg/html-to-markdown-wasm?label=WASM&color=007ec6" alt="WASM">
  </a>
  <a href="https://central.sonatype.com/artifact/dev.kreuzberg/html-to-markdown">
    <img src="https://img.shields.io/maven-central/v/dev.kreuzberg/html-to-markdown?label=Java&color=007ec6" alt="Java">
  </a>
  <a href="https://pkg.go.dev/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown">
    <img src="https://img.shields.io/badge/Go-v2.19.0-007ec6" alt="Go">
  </a>
  <a href="https://www.nuget.org/packages/KreuzbergDev.HtmlToMarkdown/">
    <img src="https://img.shields.io/nuget/v/KreuzbergDev.HtmlToMarkdown?label=C%23&color=007ec6" alt="C#">
  </a>
  <a href="https://packagist.org/packages/goldziher/html-to-markdown">
    <img src="https://img.shields.io/packagist/v/goldziher/html-to-markdown?label=PHP&color=007ec6" alt="PHP">
  </a>
  <a href="https://rubygems.org/gems/html-to-markdown">
    <img src="https://img.shields.io/gem/v/html-to-markdown?label=Ruby&color=007ec6" alt="Ruby">
  </a>
  <a href="https://hex.pm/packages/html_to_markdown">
    <img src="https://img.shields.io/hexpm/v/html_to_markdown?label=Elixir&color=007ec6" alt="Elixir">
  </a>

  <!-- Project Info -->
  <a href="https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  </a>
</div>

<img width="1128" height="191" alt="html-to-markdown" src="https://github.com/user-attachments/assets/419fc06c-8313-4324-b159-4b4d3cfce5c0" />

<div align="center" style="margin-top: 20px;">
  <a href="https://discord.gg/pXxagNK2zN">
      <img height="22" src="https://img.shields.io/badge/Discord-Join%20our%20community-7289da?logo=discord&logoColor=white" alt="Discord">
  </a>
</div>


High-performance HTML to Markdown converter with C#/.NET bindings using P/Invoke to the Rust core.
Provides type-safe record-based APIs for metadata extraction, visitor patterns, and thread-safe concurrent conversion.


## Installation

```bash
dotnet add package KreuzbergDev.HtmlToMarkdown
```



Requires .NET 8.0+ SDK.

```bash
dotnet add package KreuzbergDev.HtmlToMarkdown
```




# Migration Guide: C# v2.18.x → v2.19.0

## Breaking Change: Package Owner Update

In v2.19.0, the C#/.NET package owner changed from `Goldziher` to `KreuzbergDev` to reflect the new Kreuzberg.dev organization. The package name on NuGet changed from `Goldziher.HtmlToMarkdown` to `KreuzbergDev.HtmlToMarkdown`.

### NuGet Installation Update

**Before (v2.18.x):**
```bash
dotnet add package Goldziher.HtmlToMarkdown
```

**After (v2.19.0+):**
```bash
dotnet add package KreuzbergDev.HtmlToMarkdown
```

### Project File Update

Update your `.csproj` file:

**Before:**
```xml
&lt;PackageReference Include=&#34;Goldziher.HtmlToMarkdown&#34; Version=&#34;2.18.x&#34; /&gt;
```

**After:**
```xml
&lt;PackageReference Include=&#34;KreuzbergDev.HtmlToMarkdown&#34; Version=&#34;2.19.0&#34; /&gt;
```

### Using Statement

The namespace remains unchanged. No code modifications are required:

**Before and After:**
```csharp
using HtmlToMarkdown;
```

### Code Migration Example

**Before (v2.18.x):**
```csharp
using HtmlToMarkdown;

var html = &#34;&lt;h1&gt;Hello World&lt;/h1&gt;&lt;p&gt;This is a paragraph.&lt;/p&gt;&#34;;

try
{
    var markdown = HtmlToMarkdownConverter.Convert(html);
    Console.WriteLine(markdown);
}
catch (HtmlToMarkdownException ex)
{
    Console.Error.WriteLine($&#34;Conversion failed: {ex.Message}&#34;);
}
```

**After (v2.19.0+):**
```csharp
using HtmlToMarkdown;

var html = &#34;&lt;h1&gt;Hello World&lt;/h1&gt;&lt;p&gt;This is a paragraph.&lt;/p&gt;&#34;;

try
{
    var markdown = HtmlToMarkdownConverter.Convert(html);
    Console.WriteLine(markdown);
}
catch (HtmlToMarkdownException ex)
{
    Console.Error.WriteLine($&#34;Conversion failed: {ex.Message}&#34;);
}
```

### Metadata Extraction Update

The API and namespace remain the same:

**Before:**
```csharp
using HtmlToMarkdown;

var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);
var doc = result.Metadata.Document;
```

**After:**
```csharp
using HtmlToMarkdown;

var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);
var doc = result.Metadata.Document;
```

### Visitor Pattern Update

The visitor API remains unchanged:

**Before:**
```csharp
using HtmlToMarkdown;
using HtmlToMarkdown.Visitor;

var visitor = new MyVisitor();
var markdown = HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);
```

**After:**
```csharp
using HtmlToMarkdown;
using HtmlToMarkdown.Visitor;

var visitor = new MyVisitor();
var markdown = HtmlToMarkdownConverter.ConvertWithVisitor(html, visitor);
```

## Summary of Changes

- NuGet package renamed from `Goldziher.HtmlToMarkdown` to `KreuzbergDev.HtmlToMarkdown`
- Using statements and C# namespaces remain unchanged
- All APIs are identical
- No code changes required beyond updating the package reference
- Full backward compatibility after package update




## Performance Snapshot

Apple M4 • Real Wikipedia documents • `Convert()` (C# / .NET)

| Document | Size | Ops/sec | Throughput |
| -------- | ---- | ------- | ---------- |
| Lists (Timeline) | 129KB | 3,111 | 392.9 MB/s |
| Tables (Countries) | 360KB | 853 | 300.1 MB/s |
| Mixed (Python) | 656KB | 456 | 292.3 MB/s |


See [Performance Guide](../../examples/performance/) for detailed benchmarks.


## Quick Start

Basic conversion:

```csharp
using HtmlToMarkdown;

var html = &#34;&lt;h1&gt;Hello World&lt;/h1&gt;&lt;p&gt;This is a paragraph.&lt;/p&gt;&#34;;

try
{
    var markdown = HtmlToMarkdownConverter.Convert(html);
    Console.WriteLine(markdown);
}
catch (HtmlToMarkdownException ex)
{
    Console.Error.WriteLine($&#34;Conversion failed: {ex.Message}&#34;);
}
```



With conversion options:

```csharp
using HtmlToMarkdown;

try
{
    string html = &#34;&#34;&#34;
        &lt;html&gt;
        &lt;head&gt;
            &lt;title&gt;My Article&lt;/title&gt;
            &lt;meta name=&#34;description&#34; content=&#34;An interesting read&#34;&gt;
            &lt;meta name=&#34;author&#34; content=&#34;Jane Doe&#34;&gt;
            &lt;meta property=&#34;og:image&#34; content=&#34;image.jpg&#34;&gt;
        &lt;/head&gt;
        &lt;body&gt;
            &lt;h1&gt;Welcome&lt;/h1&gt;
            &lt;a href=&#34;https://example.com&#34;&gt;Link&lt;/a&gt;
            &lt;img src=&#34;image.jpg&#34; alt=&#34;Featured image&#34;&gt;
        &lt;/body&gt;
        &lt;/html&gt;
        &#34;&#34;&#34;;

    var result = HtmlToMarkdownConverter.ConvertWithMetadata(html);

    // Access document metadata
    var doc = result.Metadata.Document;
    if (doc.Title != null)
    {
        Console.WriteLine($&#34;Title: {doc.Title}&#34;);
    }
    if (doc.Author != null)
    {
        Console.WriteLine($&#34;Author: {doc.Author}&#34;);
    }

    // Access Open Graph metadata
    if (doc.OpenGraph != null)
    {
        foreach (var (key, value) in doc.OpenGraph)
        {
            Console.WriteLine($&#34;OG {key}: {value}&#34;);
        }
    }

    // Count extracted elements
    Console.WriteLine($&#34;Headers: {result.Metadata.Headers.Count}&#34;);
    Console.WriteLine($&#34;Links: {result.Metadata.Links.Count}&#34;);
    Console.WriteLine($&#34;Images: {result.Metadata.Images.Count}&#34;);

    // Print markdown output
    Console.WriteLine($&#34;\nMarkdown:\n{result.Markdown}&#34;);
}
catch (HtmlToMarkdownException ex)
{
    Console.Error.WriteLine($&#34;Conversion failed: {ex.Message}&#34;);
}
catch (JsonException ex)
{
    Console.Error.WriteLine($&#34;Metadata parsing failed: {ex.Message}&#34;);
}
```






## API Reference

### Core Functions


**`Convert(string html, ConversionOptions? options = null) : string`**

Basic HTML-to-Markdown conversion. Fast and simple.

**`ConvertWithMetadata(string html, ConversionOptions? options = null, MetadataConfig? config = null) : (string markdown, MetadataResult metadata)`**

Extract Markdown plus metadata in a single pass. See [Metadata Extraction Guide](../../examples/metadata-extraction/).

**`ConvertWithInlineImages(string html, InlineImageConfig? config = null) : (string markdown, InlineImageData[] images, string[] warnings)`**

Extract base64-encoded inline images with metadata.



### Options

**`ConversionOptions`** – Key configuration fields:
- `heading_style`: Heading format (`"underlined"` | `"atx"` | `"atx_closed"`) — default: `"underlined"`
- `list_indent_width`: Spaces per indent level — default: `2`
- `bullets`: Bullet characters cycle — default: `"*+-"`
- `wrap`: Enable text wrapping — default: `false`
- `wrap_width`: Wrap at column — default: `80`
- `code_language`: Default fenced code block language — default: none
- `extract_metadata`: Embed metadata as YAML frontmatter — default: `false`

**`MetadataConfig`** – Selective metadata extraction:
- `extract_headers`: h1-h6 elements — default: `true`
- `extract_links`: Hyperlinks — default: `true`
- `extract_images`: Image elements — default: `true`
- `extract_structured_data`: JSON-LD, Microdata, RDFa — default: `true`
- `max_structured_data_size`: Size limit in bytes — default: `100KB`






## Examples

- [Visitor Pattern Guide](../../examples/visitor-pattern/)
- [Metadata Extraction Guide](../../examples/metadata-extraction/)
- [Performance Guide](../../examples/performance/)

## Links

- **GitHub:** [github.com/kreuzberg-dev/html-to-markdown](https://github.com/kreuzberg-dev/html-to-markdown)

- **NuGet:** [nuget.org/packages/KreuzbergDev.HtmlToMarkdown](https://www.nuget.org/packages/KreuzbergDev.HtmlToMarkdown/)

- **Kreuzberg Ecosystem:** [kreuzberg.dev](https://kreuzberg.dev)
- **Discord:** [discord.gg/pXxagNK2zN](https://discord.gg/pXxagNK2zN)

## Contributing

We welcome contributions! Please see our [Contributing Guide](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/CONTRIBUTING.md) for details on:

- Setting up the development environment
- Running tests locally
- Submitting pull requests
- Reporting issues

All contributions must follow our code quality standards (enforced via pre-commit hooks):

- Proper test coverage (Rust 95%+, language bindings 80%+)
- Formatting and linting checks
- Documentation for public APIs

## License

MIT License – see [LICENSE](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE).

## Support

If you find this library useful, consider [sponsoring the project](https://github.com/sponsors/kreuzberg-dev).

Have questions or run into issues? We're here to help:

- **GitHub Issues:** [github.com/kreuzberg-dev/html-to-markdown/issues](https://github.com/kreuzberg-dev/html-to-markdown/issues)
- **Discussions:** [github.com/kreuzberg-dev/html-to-markdown/discussions](https://github.com/kreuzberg-dev/html-to-markdown/discussions)
- **Discord Community:** [discord.gg/pXxagNK2zN](https://discord.gg/pXxagNK2zN)
