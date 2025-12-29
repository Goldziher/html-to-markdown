# C# Basic Usage

Convert HTML to Markdown with the C# bindings.

## Simple Conversion

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

## Installation

```bash
dotnet add package KreuzbergDev.HtmlToMarkdown
```

## API Reference

### `HtmlToMarkdownConverter.Convert(string html)`

Converts HTML to Markdown. Throws `HtmlToMarkdownException` on error.

### `HtmlToMarkdownConverter.GetVersion()`

Returns the library version string.
