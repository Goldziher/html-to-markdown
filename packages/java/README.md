# html-to-markdown (Java)

High-performance HTML to Markdown converter with Rust core and Java Panama FFI bindings.

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

## Features

- **High Performance**: 60-80x faster than pure Java implementations
- **Modern FFI**: Uses Java's Foreign Function & Memory API (Project Panama)
- **CommonMark Compliant**: Follows CommonMark specification by default
- **Zero Dependencies**: No external Java dependencies (JUnit only for testing)
- **Thread Safe**: Safe for concurrent use across multiple threads

## Requirements

- **Java 25+** (uses Foreign Function & Memory API)
- **Rust toolchain** (for building the native library)
- **Maven 3.8+** (install via Homebrew, apt, choco, etc.)

## Building

### 1. Build the native library

```bash
# From repository root
cargo build --release -p html-to-markdown-ffi
```

This creates `target/release/libhtml_to_markdown_ffi.{dylib|so|dll}` depending on your platform.

### 2. Build the Java package

```bash
mvn -f packages/java/pom.xml clean package
```

The Maven build is configured to:
- Automatically build the Rust FFI library during `generate-sources` phase
- Compile Java 25+ code with `--enable-preview` flag
- Run tests with native access enabled

## Installation

### Maven

```xml
<dependency>
    <groupId>io.github.goldziher</groupId>
    <artifactId>html-to-markdown</artifactId>
    <version>2.7.3</version>
    <classifier>linux</classifier> <!-- or macos, windows -->
</dependency>
```

### Gradle (Kotlin DSL)

```kotlin
dependencies {
    implementation("io.github.goldziher:html-to-markdown:2.7.3:linux") // or macos, windows
}
```

### Gradle (Groovy)

```groovy
dependencies {
    implementation 'io.github.goldziher:html-to-markdown:2.7.3:linux' // or macos, windows
}
```

**Note**: Choose the classifier based on your target platform: `linux`, `macos`, or `windows`.

## Usage

### Basic Example (Java)

```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;

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

### Basic Example (Kotlin)

```kotlin
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown

fun main() {
    val html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>"
    val markdown = HtmlToMarkdown.convert(html)
    println(markdown)
    // Output:
    // # Hello World
    //
    // This is a **test**.
}
```

### Converting Complex HTML (Java)

```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;

public class ComplexExample {
    public static void main(String[] args) {
        String html = """
            <div>
                <h2>Features</h2>
                <ul>
                    <li>Fast <em>Rust</em> core</li>
                    <li>CommonMark compliant</li>
                    <li><code>Zero</code> dependencies</li>
                </ul>
                <table>
                    <tr>
                        <th>Language</th>
                        <th>Speed</th>
                    </tr>
                    <tr>
                        <td>Java</td>
                        <td>Fast</td>
                    </tr>
                </table>
            </div>
            """;

        String markdown = HtmlToMarkdown.convert(html);
        System.out.println(markdown);
    }
}
```

### Converting Complex HTML (Kotlin)

```kotlin
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown

fun main() {
    val html = """
        <div>
            <h2>Features</h2>
            <ul>
                <li>Fast <em>Rust</em> core</li>
                <li>CommonMark compliant</li>
                <li><code>Zero</code> dependencies</li>
            </ul>
            <table>
                <tr>
                    <th>Language</th>
                    <th>Speed</th>
                </tr>
                <tr>
                    <td>Kotlin</td>
                    <td>Fast</td>
                </tr>
            </table>
        </div>
    """.trimIndent()

    val markdown = HtmlToMarkdown.convert(html)
    println(markdown)
}
```

### Processing Multiple Documents (Java)

```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

public class BatchProcessing {
    public static void main(String[] args) {
        List<String> htmlDocuments = List.of(
            "<h1>Document 1</h1><p>Content</p>",
            "<h1>Document 2</h1><p>More content</p>",
            "<h1>Document 3</h1><p>Even more</p>"
        );

        // Thread-safe: can be called concurrently
        List<String> markdownResults = htmlDocuments.stream()
            .map(HtmlToMarkdown::convert)
            .collect(Collectors.toList());

        markdownResults.forEach(System.out::println);
    }
}
```

### Processing Multiple Documents (Kotlin)

```kotlin
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown
import kotlinx.coroutines.*

fun main() = runBlocking {
    val htmlDocuments = listOf(
        "<h1>Document 1</h1><p>Content</p>",
        "<h1>Document 2</h1><p>More content</p>",
        "<h1>Document 3</h1><p>Even more</p>"
    )

    // Thread-safe: can be called concurrently
    val markdownResults = htmlDocuments.map { html ->
        async(Dispatchers.Default) {
            HtmlToMarkdown.convert(html)
        }
    }.awaitAll()

    markdownResults.forEach(::println)
}
```

### Error Handling (Java)

```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;

public class ErrorHandlingExample {
    public static void main(String[] args) {
        String html = "<h1>Example</h1>";

        try {
            String markdown = HtmlToMarkdown.convert(html);
            System.out.println(markdown);
        } catch (HtmlToMarkdown.ConversionException e) {
            System.err.println("Conversion failed: " + e.getMessage());
        }
    }
}
```

### Error Handling (Kotlin)

```kotlin
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown

fun main() {
    val html = "<h1>Example</h1>"

    runCatching {
        HtmlToMarkdown.convert(html)
    }.onSuccess { markdown ->
        println(markdown)
    }.onFailure { error ->
        when (error) {
            is HtmlToMarkdown.ConversionException -> {
                System.err.println("Conversion failed: ${error.message}")
            }
            else -> throw error
        }
    }
}
```

### Get Library Version

```java
// Java
String version = HtmlToMarkdown.getVersion();
System.out.println("html-to-markdown version: " + version);

// Kotlin
val version = HtmlToMarkdown.getVersion()
println("html-to-markdown version: $version")
```

## Metadata Extraction

The `convertWithMetadata()` method extracts comprehensive document metadata in a single pass while converting HTML to Markdown. This is useful for SEO analysis, content management systems, web crawlers, and document processors.

### Method Signature

```java
/**
 * Convert HTML to Markdown with metadata extraction.
 * <p>
 * This method converts HTML to Markdown while extracting document metadata
 * such as titles, headers, links, images, and structured data.
 *
 * @param html the HTML string to convert
 * @return a {@code MetadataExtraction} containing both markdown and metadata
 * @throws NullPointerException if html is null
 * @throws ConversionException if the conversion fails
 *
 * @since 2.13.0
 */
public static MetadataExtraction convertWithMetadata(String html)
```

### Metadata Record Types

The metadata extraction API uses Java records for type-safe access. All metadata is extracted into an `ExtendedMetadata` object containing:

#### `ExtendedMetadata`

Top-level metadata container with all extracted information:

```java
public record ExtendedMetadata(
    DocumentMetadata document,
    List<HeaderMetadata> headers,
    List<LinkMetadata> links,
    List<ImageMetadata> images,
    List<StructuredData> structuredData
)
```

#### `DocumentMetadata`

Document-level metadata from `<head>` and top-level elements:

```java
public record DocumentMetadata(
    String title,              // <title> content
    String description,        // meta[name="description"]
    List<String> keywords,     // meta[name="keywords"], split on commas
    String author,             // meta[name="author"]
    String canonicalUrl,       // <link rel="canonical">
    String baseHref,           // <base href="">
    String language,           // lang attribute
    String textDirection,      // dir attribute (ltr, rtl, auto)
    Map<String, String> openGraph,    // og:* properties
    Map<String, String> twitterCard,  // twitter:* properties
    Map<String, String> metaTags      // Other meta tags
)
```

#### `HeaderMetadata`

Header element metadata with hierarchy tracking:

```java
public record HeaderMetadata(
    int level,        // Header level (1 for h1, 6 for h6)
    String text,      // Normalized text content
    String id,        // HTML id attribute
    int depth,        // Document tree depth
    int htmlOffset    // Byte offset in original HTML
)
```

#### `LinkMetadata`

Hyperlink metadata with classification:

```java
public record LinkMetadata(
    String href,                      // href attribute value
    String text,                      // Link text content
    String title,                     // title attribute
    String linkType,                  // Classification (anchor, internal, external, email, phone, other)
    List<String> rel,                 // rel attribute values
    Map<String, String> attributes    // Additional HTML attributes
)
```

#### `ImageMetadata`

Image metadata with source and dimensions:

```java
public record ImageMetadata(
    String src,                       // Image source (URL, data URI, or SVG identifier)
    String alt,                       // alt attribute (accessibility)
    String title,                     // title attribute
    int[] dimensions,                 // [width, height] if available
    String imageType,                 // Classification (data_uri, inline_svg, external, relative)
    Map<String, String> attributes    // Additional HTML attributes
)
```

#### `StructuredData`

Machine-readable structured data blocks:

```java
public record StructuredData(
    String dataType,   // json_ld, microdata, rdfa
    String rawJson,    // Raw JSON string (for JSON-LD)
    String schemaType  // Schema type (e.g., "Article", "Event")
)
```

#### `MetadataExtraction`

Result container combining markdown and extracted metadata:

```java
public record MetadataExtraction(
    String markdown,
    ExtendedMetadata metadata
)
```

### Metadata Examples

#### Basic Metadata Extraction with Exception Handling

Extract and access document metadata:

```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.metadata.MetadataExtraction;

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

#### Processing Headers with Hierarchy

Extract and traverse document structure:

```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.metadata.HeaderMetadata;

public class HeaderTraversalExample {
    public static void main(String[] args) {
        String html = """
            <html><body>
                <h1>Main Title</h1>
                <h2>Section One</h2>
                <h3>Subsection</h3>
                <h2>Section Two</h2>
            </body></html>
            """;

        try {
            var result = HtmlToMarkdown.convertWithMetadata(html);

            System.out.println("Document Structure:");
            for (HeaderMetadata header : result.metadata().headers()) {
                String indent = "  ".repeat(header.level() - 1);
                System.out.print(indent + "- Level " + header.level() + ": " + header.text());
                if (header.id() != null) {
                    System.out.print(" (#" + header.id() + ")");
                }
                System.out.println();
            }
        } catch (HtmlToMarkdown.ConversionException e) {
            System.err.println("Failed: " + e.getMessage());
        }
    }
}
```

#### Analyzing Links with Type Classification

Extract and filter links by type:

```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.metadata.LinkMetadata;
import java.util.List;
import java.util.stream.Collectors;

public class LinkAnalysisExample {
    public static void main(String[] args) {
        String html = """
            <html><body>
                <a href="https://external.com">External Site</a>
                <a href="/internal">Internal Page</a>
                <a href="#section">Anchor Link</a>
                <a href="mailto:test@example.com">Email</a>
                <a href="tel:+1234567890">Phone</a>
                <a href="https://external.com" rel="nofollow">Nofollow Link</a>
            </body></html>
            """;

        try {
            var result = HtmlToMarkdown.convertWithMetadata(html);

            // Group links by type
            var linksByType = result.metadata().links().stream()
                .collect(Collectors.groupingBy(LinkMetadata::linkType));

            // Process external links
            var external = linksByType.getOrDefault("external", List.of());
            if (!external.isEmpty()) {
                System.out.println("External Links: " + external.size());
                external.forEach(link ->
                    System.out.println("  - " + link.text() + ": " + link.href())
                );
            }

            // Find nofollow links
            var nofollow = result.metadata().links().stream()
                .filter(link -> link.rel().contains("nofollow"))
                .collect(Collectors.toList());

            if (!nofollow.isEmpty()) {
                System.out.println("\nNofollow Links: " + nofollow.size());
                nofollow.forEach(link ->
                    System.out.println("  - " + link.text() + ": " + link.href())
                );
            }
        } catch (HtmlToMarkdown.ConversionException e) {
            System.err.println("Failed: " + e.getMessage());
        }
    }
}
```

### Panama FFM Integration Notes

The `convertWithMetadata()` method uses Java's Foreign Function & Memory API (Panama) for FFI integration:

1. **Memory Management**: Uses `Arena` for safe, automatic memory lifecycle management
2. **String Conversion**: Transparently converts between Java strings and C strings
3. **JSON Parsing**: Metadata JSON is deserialized to typed Java records using Jackson
4. **Exception Handling**: All FFI errors are wrapped in `ConversionException` for idiomatic error handling
5. **Thread Safety**: The Panama FFI calls are thread-safe through proper memory isolation

No manual pointer management is required - `Arena` handles all native memory cleanup.

## Running Tests

```bash
mvn -f packages/java/pom.xml test
```

Tests require:
- The native library to be built and accessible via `java.library.path`
- JVM flags: `--enable-preview --enable-native-access=ALL-UNNAMED`

## Running with Java

When running your application, ensure the native library is in the library path:

### macOS

```bash
java --enable-preview --enable-native-access=ALL-UNNAMED \
  -Djava.library.path=../../target/release \
  -jar your-app.jar
```

### Linux

```bash
java --enable-preview --enable-native-access=ALL-UNNAMED \
  -Djava.library.path=../../target/release \
  -jar your-app.jar
```

### Windows

```powershell
java --enable-preview --enable-native-access=ALL-UNNAMED `
  -Djava.library.path=..\..\target\release `
  -jar your-app.jar
```

## Architecture

The Java bindings use a two-layer architecture:

1. **`HtmlToMarkdownFFI`** (package-private): Low-level Foreign Function Interface
   - Direct bindings to C FFI functions using Panama's `MethodHandle`
   - Handles memory management and string conversion
   - Uses `Arena` for safe memory lifecycle management

2. **`HtmlToMarkdown`** (public API): High-level wrapper
   - Ergonomic Java API
   - Automatic resource cleanup
   - Type-safe exception handling

## Performance

The Rust-backed implementation provides significant performance improvements:

| Document Type          | Size   | Ops/sec  | Throughput |
| ---------------------- | ------ | -------- | ---------- |
| Lists (Timeline)       | 129 KB | 1,001    | 126.4 MB/s |
| Tables (Countries)     | 360 KB | 277      | 97.4 MB/s  |
| Medium (Python)        | 656 KB | 138      | 88.1 MB/s  |
| Large (Rust)           | 567 KB | 151      | 83.5 MB/s  |
| Small (Intro)          | 463 KB | 184      | 83.2 MB/s  |
| HOCR German PDF        | 44 KB  | 2,069    | 88.3 MB/s  |
| HOCR Invoice           | 4 KB   | 15,067   | 61.7 MB/s  |
| HOCR Embedded Tables   | 37 KB  | 2,069    | 75.1 MB/s  |

## Supported Platforms

- **macOS**: arm64, x86_64
- **Linux**: x86_64 (gnu, musl)
- **Windows**: x86_64 (MSVC)

## Thread Safety

`HtmlToMarkdown.convert()` is thread-safe and can be called concurrently from multiple threads. See the "Processing Multiple Documents" examples above for concurrent usage patterns.

## License

MIT License - see LICENSE file in repository root.

## Contributing

See the main repository [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Links

- [Repository](https://github.com/Goldziher/html-to-markdown)
- [Issue Tracker](https://github.com/Goldziher/html-to-markdown/issues)
- [Changelog](../../CHANGELOG.md)
