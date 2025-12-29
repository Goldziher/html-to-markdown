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
<PackageReference Include="Goldziher.HtmlToMarkdown" Version="2.18.x" />
```

**After:**
```xml
<PackageReference Include="KreuzbergDev.HtmlToMarkdown" Version="2.19.0" />
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

**After (v2.19.0+):**
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
