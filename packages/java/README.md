# html-to-markdown (Java)

High-performance HTML to Markdown converter with Rust core and Java Panama FFI bindings.

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

## Usage

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

### Get Library Version

```java
String version = HtmlToMarkdown.getVersion();
System.out.println("html-to-markdown version: " + version);
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

| Document Type       | Size  | Throughput | Speedup vs Pure Java |
| ------------------- | ----- | ---------- | -------------------- |
| Lists (Timeline)    | 129KB | 208 MB/s   | **~80x**             |
| Tables (Countries)  | 360KB | 178 MB/s   | **~70x**             |
| Mixed (Python wiki) | 656KB | 144 MB/s   | **~60x**             |

## Supported Platforms

- **macOS**: arm64, x86_64
- **Linux**: x86_64 (gnu, musl)
- **Windows**: x86_64 (MSVC)

## Thread Safety

`HtmlToMarkdown.convert()` is thread-safe and can be called concurrently from multiple threads.

## Error Handling

```java
try {
    String markdown = HtmlToMarkdown.convert(html);
} catch (HtmlToMarkdown.ConversionException e) {
    System.err.println("Conversion failed: " + e.getMessage());
}
```

## License

MIT License - see LICENSE file in repository root.

## Contributing

See the main repository [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Links

- [Repository](https://github.com/Goldziher/html-to-markdown)
- [Issue Tracker](https://github.com/Goldziher/html-to-markdown/issues)
- [Changelog](../../CHANGELOG.md)
