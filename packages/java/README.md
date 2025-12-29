# html-to-markdown (Java)

High-performance HTML to Markdown converter with Rust core and Java Panama FFI bindings.

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg?logo=rust&label=crates.io)](https://crates.io/crates/html-to-markdown-rs)
[![npm (node)](https://img.shields.io/npm/v/html-to-markdown-node.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://img.shields.io/npm/v/html-to-markdown-wasm.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://img.shields.io/pypi/v/html-to-markdown.svg?logo=pypi)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/KreuzbergDev.HtmlToMarkdown.svg)](https://www.nuget.org/packages/KreuzbergDev.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/dev.kreuzberg/html-to-markdown.svg)](https://central.sonatype.com/artifact/dev.kreuzberg/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown.svg)](https://pkg.go.dev/github.com/kreuzberg-dev/html-to-markdown/packages/go/v2/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/kreuzberg-dev/html-to-markdown/blob/main/LICENSE)
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
    <groupId>dev.kreuzberg</groupId>
    <artifactId>html-to-markdown</artifactId>
    <version>2.19.0</version>
    <classifier>linux</classifier> <!-- or macos, windows -->
</dependency>
```

### Gradle (Kotlin DSL)

```kotlin
dependencies {
    implementation("dev.kreuzberg:html-to-markdown:2.19.0:linux") // or macos, windows
}
```

### Gradle (Groovy)

```groovy
dependencies {
    implementation 'dev.kreuzberg:html-to-markdown:2.19.0:linux' // or macos, windows
}
```

**Note**: Choose the classifier based on your target platform: `linux`, `macos`, or `windows`.

## Usage

### Basic Example (Java)

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

### Basic Example (Kotlin)

```kotlin
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown

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
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;

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
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
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
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;

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

#### Processing Headers with Hierarchy

Extract and traverse document structure:

```java
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.metadata.HeaderMetadata;

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
import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.metadata.LinkMetadata;
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

## Visitor Pattern

The visitor pattern enables fine-grained control over HTML-to-Markdown conversion by allowing custom logic to intercept and process specific HTML elements during traversal. This is useful for filtering content, customizing output format, extracting metadata, analytics, and content analysis.

### Overview

The visitor pattern provides a callback-based API that integrates with Panama FFI to invoke Java methods as the converter traverses the DOM tree. Each visitor method corresponds to an HTML element type.

### Core Interfaces

#### `Visitor` Interface

The main visitor interface with 40+ optional callback methods:

```java
public interface Visitor {
    // Generic element callbacks
    default VisitResult visitElementStart(NodeContext ctx) { ... }
    default VisitResult visitElementEnd(NodeContext ctx, String output) { ... }

    // Text nodes
    default VisitResult visitText(NodeContext ctx, String text) { ... }

    // Links and images
    default VisitResult visitLink(NodeContext ctx, String href, String text, String title) { ... }
    default VisitResult visitImage(NodeContext ctx, String src, String alt, String title) { ... }

    // Headings
    default VisitResult visitHeading(NodeContext ctx, int level, String text, String id) { ... }

    // Code
    default VisitResult visitCodeBlock(NodeContext ctx, String lang, String code) { ... }
    default VisitResult visitCodeInline(NodeContext ctx, String code) { ... }

    // Lists
    default VisitResult visitListStart(NodeContext ctx, boolean ordered) { ... }
    default VisitResult visitListItem(NodeContext ctx, boolean ordered, String marker, String text) { ... }
    default VisitResult visitListEnd(NodeContext ctx, boolean ordered, String output) { ... }

    // Tables
    default VisitResult visitTableStart(NodeContext ctx) { ... }
    default VisitResult visitTableRow(NodeContext ctx, List<String> cells, boolean isHeader) { ... }
    default VisitResult visitTableEnd(NodeContext ctx, String output) { ... }

    // And many more (blockquotes, formatting, forms, media, semantic HTML5, etc.)
}
```

#### `VisitResult` Sealed Interface

Controls how the converter proceeds after a callback:

```java
public sealed interface VisitResult {
    // Continue with default conversion
    record Continue() implements VisitResult { }

    // Replace with custom markdown
    record Custom(String customOutput) implements VisitResult { }

    // Skip element and children
    record Skip() implements VisitResult { }

    // Preserve original HTML
    record PreserveHtml() implements VisitResult { }

    // Report error and stop
    record Error(String errorMessage) implements VisitResult { }
}
```

#### `NodeContext` Record

Provides metadata about the element being visited:

```java
public record NodeContext(
    NodeType nodeType,           // Element type classification
    String tagName,              // HTML tag name ("div", "h1", etc.)
    List<Attribute> attributes,  // HTML attributes
    int depth,                   // DOM depth (0 = root)
    int indexInParent,           // Position among siblings
    String parentTag,            // Parent element tag
    boolean isInline             // Inline vs block element
) {
    public boolean isHeading() { ... }
    public boolean isTextNode() { ... }
    public String getAttributeValue(String name) { ... }
    public boolean hasAttribute(String name) { ... }
}
```

### Usage Examples

#### Example 1: Filter External Links

```java
public class ExternalLinkFilter implements Visitor {
    private static final String INTERNAL_DOMAIN = "example.com";

    @Override
    public VisitResult visitLink(NodeContext ctx, String href, String text, String title) {
        if (href != null && !href.startsWith("/") && !href.contains(INTERNAL_DOMAIN)) {
            // Skip external links
            return VisitResult.Skip.INSTANCE;
        }
        return VisitResult.Continue.INSTANCE;
    }
}

// Usage
String html = "<p><a href=\"https://external.com\">Bad</a> <a href=\"/page\">Good</a></p>";
Visitor visitor = new ExternalLinkFilter();
// String result = HtmlToMarkdown.convertWithVisitor(html, visitor);
```

#### Example 2: Custom Heading Format

```java
public class CustomHeadingFormat implements Visitor {
    @Override
    public VisitResult visitHeading(NodeContext ctx, int level, String text, String id) {
        // Custom format: >>> Heading <<<
        String custom = ">".repeat(level) + " " + text + " " + "<".repeat(level);
        return new VisitResult.Custom(custom);
    }
}
```

#### Example 3: Document Analytics

```java
public class DocumentAnalytics implements Visitor {
    private int linkCount = 0;
    private int imageCount = 0;
    private Map<Integer, Integer> headingLevels = new TreeMap<>();

    @Override
    public VisitResult visitLink(NodeContext ctx, String href, String text, String title) {
        linkCount++;
        return VisitResult.Continue.INSTANCE;
    }

    @Override
    public VisitResult visitImage(NodeContext ctx, String src, String alt, String title) {
        imageCount++;
        return VisitResult.Continue.INSTANCE;
    }

    @Override
    public VisitResult visitHeading(NodeContext ctx, int level, String text, String id) {
        headingLevels.put(level, headingLevels.getOrDefault(level, 0) + 1);
        return VisitResult.Continue.INSTANCE;
    }

    public void printStats() {
        System.out.println("Links: " + linkCount);
        System.out.println("Images: " + imageCount);
        System.out.println("Headings by level: " + headingLevels);
    }
}
```

#### Example 4: Remove Script and Style Tags

```java
public class ContentSanitizer implements Visitor {
    @Override
    public VisitResult visitCustomElement(NodeContext ctx, String tagName, String html) {
        if ("script".equals(tagName) || "style".equals(tagName)) {
            return VisitResult.Skip.INSTANCE;
        }
        return VisitResult.Continue.INSTANCE;
    }
}
```

#### Example 5: Attribute Extraction

```java
public class AttributeCollector implements Visitor {
    private Map<String, Integer> classNames = new HashMap<>();

    @Override
    public VisitResult visitElementStart(NodeContext ctx) {
        String className = ctx.getAttributeValue("class");
        if (className != null && !className.isEmpty()) {
            classNames.put(className, classNames.getOrDefault(className, 0) + 1);
        }
        return VisitResult.Continue.INSTANCE;
    }

    public Map<String, Integer> getClassFrequency() {
        return classNames;
    }
}
```

### All Visitor Methods

The `Visitor` interface provides the following callback methods (all with default implementations returning `Continue`):

#### Generic Element Hooks
- `visitElementStart(NodeContext)` - Before processing any element
- `visitElementEnd(NodeContext, String)` - After processing element with default output

#### Text Nodes
- `visitText(NodeContext, String)` - Text node (called frequently, optimize for performance)

#### Links and Media
- `visitLink(NodeContext, String, String, String)` - `<a>` elements
- `visitImage(NodeContext, String, String, String)` - `<img>` elements
- `visitAudio(NodeContext, String)` - `<audio>` elements
- `visitVideo(NodeContext, String)` - `<video>` elements
- `visitIframe(NodeContext, String)` - `<iframe>` elements

#### Headings
- `visitHeading(NodeContext, int, String, String)` - `<h1>` through `<h6>`

#### Code
- `visitCodeBlock(NodeContext, String, String)` - `<pre><code>` blocks
- `visitCodeInline(NodeContext, String)` - `<code>` inline

#### Lists
- `visitListStart(NodeContext, boolean)` - `<ul>` / `<ol>` start
- `visitListItem(NodeContext, boolean, String, String)` - `<li>` elements
- `visitListEnd(NodeContext, boolean, String)` - `<ul>` / `<ol>` end
- `visitDefinitionListStart(NodeContext)` - `<dl>` start
- `visitDefinitionTerm(NodeContext, String)` - `<dt>` elements
- `visitDefinitionDescription(NodeContext, String)` - `<dd>` elements
- `visitDefinitionListEnd(NodeContext, String)` - `<dl>` end

#### Tables
- `visitTableStart(NodeContext)` - `<table>` start
- `visitTableRow(NodeContext, List<String>, boolean)` - `<tr>` rows
- `visitTableEnd(NodeContext, String)` - `</table>` end

#### Blockquotes
- `visitBlockquote(NodeContext, String, int)` - `<blockquote>` elements

#### Inline Formatting
- `visitStrong(NodeContext, String)` - `<strong>` / `<b>`
- `visitEmphasis(NodeContext, String)` - `<em>` / `<i>`
- `visitStrikethrough(NodeContext, String)` - `<s>` / `<del>` / `<strike>`
- `visitUnderline(NodeContext, String)` - `<u>` / `<ins>`
- `visitSubscript(NodeContext, String)` - `<sub>`
- `visitSuperscript(NodeContext, String)` - `<sup>`
- `visitMark(NodeContext, String)` - `<mark>`

#### Breaks
- `visitLineBreak(NodeContext)` - `<br>`
- `visitHorizontalRule(NodeContext)` - `<hr>`

#### Forms
- `visitForm(NodeContext, String, String)` - `<form>` elements
- `visitInput(NodeContext, String, String, String)` - `<input>` elements
- `visitButton(NodeContext, String)` - `<button>` elements

#### Semantic HTML5
- `visitDetails(NodeContext, boolean)` - `<details>` elements
- `visitSummary(NodeContext, String)` - `<summary>` elements
- `visitFigureStart(NodeContext)` - `<figure>` start
- `visitFigcaption(NodeContext, String)` - `<figcaption>` elements
- `visitFigureEnd(NodeContext, String)` - `</figure>` end

#### Custom Elements
- `visitCustomElement(NodeContext, String, String)` - Web components and unknown tags

### NodeType Enum

All supported HTML element types:

```
TEXT, ELEMENT, HEADING, PARAGRAPH, DIV, BLOCKQUOTE, PRE, HR,
LIST, LIST_ITEM, DEFINITION_LIST, DEFINITION_TERM, DEFINITION_DESCRIPTION,
TABLE, TABLE_ROW, TABLE_CELL, TABLE_HEADER, TABLE_BODY, TABLE_HEAD, TABLE_FOOT,
LINK, IMAGE, STRONG, EM, CODE, STRIKETHROUGH, UNDERLINE, SUBSCRIPT, SUPERSCRIPT,
MARK, SMALL, BR, SPAN,
ARTICLE, SECTION, NAV, ASIDE, HEADER, FOOTER, MAIN, FIGURE, FIGCAPTION, TIME, DETAILS, SUMMARY,
FORM, INPUT, SELECT, OPTION, BUTTON, TEXTAREA, LABEL, FIELDSET, LEGEND,
AUDIO, VIDEO, PICTURE, SOURCE, IFRAME, SVG, CANVAS,
RUBY, RT, RP, ABBR, KBD, SAMP, VAR, CITE, Q, DEL, INS, DATA, METER, PROGRESS, OUTPUT, TEMPLATE, SLOT,
HTML, HEAD, BODY, TITLE, META, LINK_TAG, STYLE, SCRIPT, BASE,
CUSTOM
```

### Performance Considerations

1. **Text Node Callback Frequency**: `visitText()` is called 100+ times per document. Return `Continue` quickly unless modifying text.

2. **Memory Safety**: String data in callbacks is borrowed from Rust. Copy immediately if you need to persist data beyond the callback.

3. **Thread Safety**: If sharing a visitor across threads, ensure all callback methods are thread-safe. The underlying converter is thread-safe.

4. **Avoid Blocking Operations**: Callbacks execute synchronously during traversal. Avoid I/O, network calls, or other blocking operations.

5. **State Accumulation**: Use instance fields to accumulate statistics or analyze documents without modifying output.

### Panama FFI Integration

The visitor pattern integrates with Panama FFI through:

1. **Callback Registration**: Java visitor methods are registered with the Rust converter via `VisitorBridge`
2. **Memory Marshaling**: `NodeContext` and other data types are marshaled between Java and C representations
3. **String Conversion**: Transparent UTF-8 string conversion between Java and C
4. **Result Translation**: `VisitResult` types are converted to C-compatible enum values
5. **Error Handling**: Conversion errors from visitors propagate as `ConversionException`

No manual pointer management is required - the bridge handles all FFI complexity.

## Migration Guide (v2.18.x → v2.19.0)

### Breaking Change: Package Namespace

In v2.19.0, the Java package namespace changed from `io.github.goldziher` to `dev.kreuzberg` to reflect the new Kreuzberg.dev organization.

#### Maven Dependency Update

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
</dependency>
```

#### Import Statement Updates

Update all Java import statements:

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

#### Gradle Build Updates

**Kotlin DSL - Before:**
```kotlin
implementation("io.github.goldziher:html-to-markdown:2.18.x")
```

**Kotlin DSL - After:**
```kotlin
implementation("dev.kreuzberg:html-to-markdown:2.19.0")
```

**Groovy DSL - Before:**
```groovy
implementation 'io.github.goldziher:html-to-markdown:2.18.x'
```

**Groovy DSL - After:**
```groovy
implementation 'dev.kreuzberg:html-to-markdown:2.19.0'
```

#### Summary of Changes

- All public classes moved to `dev.kreuzberg.htmltomarkdown` package
- All metadata classes moved to `dev.kreuzberg.htmltomarkdown.metadata` package
- No functional changes to the API
- Full backward compatibility after import updates

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
| Lists (Timeline)       | 129 KB | 2,308    | 291.5 MB/s |
| Tables (Countries)     | 360 KB | 773      | 272.0 MB/s |
| Medium (Python)        | 656 KB | 403      | 258.5 MB/s |
| Large (Rust)           | 567 KB | 445      | 246.1 MB/s |
| Small (Intro)          | 463 KB | 572      | 259.0 MB/s |
| HOCR German PDF        | 44 KB  | 4,757    | 202.9 MB/s |
| HOCR Invoice           | 4 KB   | 29,006   | 118.7 MB/s |
| HOCR Embedded Tables   | 37 KB  | 5,042    | 183.0 MB/s |

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

- [Repository](https://github.com/kreuzberg-dev/html-to-markdown)
- [Issue Tracker](https://github.com/kreuzberg-dev/html-to-markdown/issues)
- [Changelog](../../CHANGELOG.md)
