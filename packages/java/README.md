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


High-performance HTML to Markdown converter with Java Panama FFI bindings to the Rust core.
Uses Foreign Function & Memory API for zero-dependency, thread-safe conversion with full metadata extraction support.


## Installation

```bash
<dependency>
    <groupId>dev.kreuzberg</groupId>
    <artifactId>html-to-markdown</artifactId>
    <version>2.19.0</version>
    <classifier>linux</classifier> <!-- or macos, windows -->
</dependency>

```



Requires Java 25+ with Panama FFI support.

**Maven:**
```xml
<dependency>
    <groupId>dev.kreuzberg</groupId>
    <artifactId>html-to-markdown</artifactId>
    <version>2.19.0</version>
</dependency>
```

**Gradle (Kotlin DSL):**
```kotlin
implementation("dev.kreuzberg:html-to-markdown:2.19.0")
```




# Migration Guide: Java v2.18.x → v2.19.0

## Breaking Change: Package Namespace

In v2.19.0, the Java package namespace changed from `io.github.goldziher` to `dev.kreuzberg` to reflect the new Kreuzberg.dev organization.

### Maven Dependency Update

**Before (v2.18.x):**
```xml
<dependency>
    <groupId>io.github.goldziher</groupId>
    <artifactId>html-to-markdown</artifactId>
    <version>2.18.x</version>
</dependency>
```

**After (v2.19.0+):**
```xml
<dependency>
    <groupId>dev.kreuzberg</groupId>
    <artifactId>html-to-markdown</artifactId>
    <version>2.19.0</version>
    <classifier>linux</classifier> <!-- or macos, windows -->
</dependency>
```

### Import Statement Updates

Update all Java import statements to use the new namespace:

**Before:**
```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.metadata.*;
```

**After:**
```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.metadata.*;
```

### Gradle Build Updates

**Kotlin DSL - Before:**
```kotlin
implementation("io.github.goldziher:html-to-markdown:2.18.x")
```

**Kotlin DSL - After:**
```kotlin
implementation("dev.kreuzberg:html-to-markdown:2.19.0:linux") // or macos, windows
```

**Groovy DSL - Before:**
```groovy
implementation 'io.github.goldziher:html-to-markdown:2.18.x'
```

**Groovy DSL - After:**
```groovy
implementation 'dev.kreuzberg:html-to-markdown:2.19.0:linux' // or macos, windows
```

### Code Migration Example

**Before (v2.18.x):**
```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;

public class Example {
    public static void main(String[] args) {
        String html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>";
        String markdown = HtmlToMarkdown.convert(html);
        System.out.println(markdown);
    }
}
```

**After (v2.19.0+):**
```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;

public class Example {
    public static void main(String[] args) {
        String html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>";
        String markdown = HtmlToMarkdown.convert(html);
        System.out.println(markdown);
    }
}
```

### Metadata Extraction Update

If you use metadata extraction, update the imports as well:

**Before:**
```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.metadata.MetadataExtraction;
```

**After:**
```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.metadata.MetadataExtraction;
```

### Visitor Pattern Update

**Before:**
```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.visitor.Visitor;
```

**After:**
```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.visitor.Visitor;
```

## Summary of Changes

- All public classes moved to `dev.kreuzberg.htmltomarkdown` package
- All metadata classes moved to `dev.kreuzberg.htmltomarkdown.metadata` package
- All visitor classes moved to `dev.kreuzberg.htmltomarkdown.visitor` package
- Maven Central groupId changed from `io.github.goldziher` to `dev.kreuzberg`
- Platform classifiers (linux, macos, windows) are now required in dependency declarations
- No functional changes to the API
- Full backward compatibility after import updates




## Performance Snapshot

Apple M4 • Real Wikipedia documents • `convert()` (Java)

| Document | Size | Ops/sec | Throughput |
| -------- | ---- | ------- | ---------- |
| Lists (Timeline) | 129KB | 2,308 | 291.5 MB/s |
| Tables (Countries) | 360KB | 773 | 272.0 MB/s |
| Mixed (Python) | 656KB | 403 | 258.5 MB/s |


See [Performance Guide](../../examples/performance/) for detailed benchmarks.


## Quick Start

Basic conversion:

```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;

public class Example {
    public static void main(String[] args) {
        String html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>";
        String markdown = HtmlToMarkdown.convert(html);
        System.out.println(markdown);
        // Output:
        // # Hello World
        //
        // This is a **test**.
    }
}
```



With conversion options:

```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.metadata.MetadataExtraction;

public class MetadataExample {
    public static void main(String[] args) {
        String html = """
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

        try {
            MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

            // Access document metadata
            var doc = result.metadata().document();
            if (doc.title() != null) {
                System.out.println("Title: " + doc.title());
            }
            if (doc.author() != null) {
                System.out.println("Author: " + doc.author());
            }

            // Access Open Graph metadata
            doc.openGraph().forEach((key, value) ->
                System.out.println("OG " + key + ": " + value)
            );

            // Count extracted elements
            System.out.println("Headers: " + result.metadata().headers().size());
            System.out.println("Links: " + result.metadata().links().size());
            System.out.println("Images: " + result.metadata().images().size());

            // Print markdown output
            System.out.println("\nMarkdown:\n" + result.markdown());
        } catch (HtmlToMarkdown.ConversionException e) {
            System.err.println("Conversion failed: " + e.getMessage());
        }
    }
}
```






## API Reference

### Core Functions


**`convert(String html) : String`**
**`convert(String html, ConversionOptions options) : String`**

Basic HTML-to-Markdown conversion. Fast and simple.

**`convertWithMetadata(String html) : ConversionResult<MetadataResult>`**
**`convertWithMetadata(String html, ConversionOptions options, MetadataConfig config) : ConversionResult<MetadataResult>`**

Extract Markdown plus metadata in a single pass. See [Metadata Extraction Guide](../../examples/metadata-extraction/).

**`convertWithInlineImages(String html, InlineImageConfig config) : ConversionResult<InlineImagesResult>`**

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

- **Maven Central:** [central.sonatype.com/artifact/dev.kreuzberg/html-to-markdown](https://central.sonatype.com/artifact/dev.kreuzberg/html-to-markdown)

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
