# html-to-markdown (Java)

High-performance HTML to Markdown converter with Rust core and Java Panama FFI bindings.

[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)

## Features

- **High Performance**: 60-80x faster than pure Java implementations
- **Modern FFI**: Uses Java's Foreign Function & Memory API (Project Panama)
- **CommonMark Compliant**: Follows CommonMark specification by default
- **Zero Dependencies**: No external Java dependencies (JUnit only for testing)
- **Thread Safe**: Safe for concurrent use across multiple threads

## Requirements

- **Java 22+** (uses Foreign Function & Memory API)
- **Maven 3.x** (for building)
- **Rust toolchain** (for building the native library)

## Building

### 1. Build the native library

```bash
# From repository root
cargo build --release -p html-to-markdown-ffi
```

This creates `target/release/libhtml_to_markdown_ffi.{dylib|so|dll}` depending on your platform.

### 2. Build the Java package

```bash
cd packages/java
mvn clean package
```

The Maven build is configured to:
- Automatically build the Rust FFI library during `generate-sources` phase
- Compile Java 22+ code with `--enable-preview` flag
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

## Running Tests

```bash
mvn test
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
