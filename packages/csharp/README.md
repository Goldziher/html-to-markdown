# html-to-markdown C# Bindings

High-performance HTML to Markdown converter with C# bindings to the Rust core library.

## Installation

```bash
dotnet add package HtmlToMarkdown
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

## Testing

```bash
cd packages/csharp
dotnet test HtmlToMarkdown.Tests/HtmlToMarkdown.Tests.csproj
```

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
dotnet nuget push bin/Release/HtmlToMarkdown.2.8.0.nupkg \
    --api-key YOUR_API_KEY \
    --source https://api.nuget.org/v3/index.json
```

### 4. Verify

Check your package at: https://www.nuget.org/packages/HtmlToMarkdown/

## License

MIT
