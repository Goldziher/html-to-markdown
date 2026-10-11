```bash
{{ install_command }}
```

{% if language == 'python' %}
Requires Python 3.10+. Wheels are published for Linux, macOS, and Windows on PyPI.

**Troubleshooting on Windows:** If `pip install` fails on Python 3.14+, use `pip install --only-binary=:all: html-to-markdown` to force the prebuilt wheel. If sdist fallback is required, install Microsoft Visual Studio Build Tools with the "Desktop development with C++" workload and ensure MSVC's `link.exe` precedes GNU variants in `PATH` (check with `where link`). Update `pip` with `pip install --upgrade pip` to ensure correct wheel matching for Python 3.14.
{% elif language == 'typescript' %}

Requires Node.js 18+ or Bun. Native bindings provide superior performance.

**npm:**

```bash
pnpm add @xberg-io/html-to-markdown
```

**pnpm:**

```bash
pnpm add @xberg-io/html-to-markdown
```

**yarn:**

```bash
yarn add @xberg-io/html-to-markdown
```

**bun:**

```bash
bun add @xberg-io/html-to-markdown
```

Alternatively, use the WebAssembly version for browser/edge environments:

```bash
pnpm add @xberg-io/html-to-markdown-wasm
```

{% elif language == 'ruby' %}

Requires Ruby 3.2+ with Magnus native extension bindings. Published for Linux, macOS.
{% elif language == 'php' %}

Requires PHP 8.2+. Install the native extension via PIE:

```bash
pie install xberg-io/html-to-markdown
```

Or use Composer (requires ext-html_to_markdown):

```bash
composer require xberg-io/html-to-markdown
```

{% elif language == 'go' %}

Requires Go 1.25+. After installing the package, run `go generate` to automatically download the platform-specific FFI library:

```bash
go generate
```

This downloads the native library from GitHub releases and generates the necessary CGO flags. The library is cached in `~/.html-to-markdown/` for subsequent builds.

Alternatively, you can manually set `CGO_CFLAGS` and `CGO_LDFLAGS` environment variables if you prefer to manage the FFI library yourself.
{% elif language == 'java' %}

Requires Java 25+ with Panama FFI support.

**Maven:**

```xml
<dependency>
    <groupId>io.xberg</groupId>
    <artifactId>html-to-markdown</artifactId>
    <version>{{ version }}</version>
</dependency>
```

**Gradle (Kotlin DSL):**

```kotlin
implementation("io.xberg:html-to-markdown:{{ version }}")
```

{% elif language == 'csharp' %}

Requires .NET 8.0+ SDK.

```bash
dotnet add package XbergIo.HtmlToMarkdown
```

{% elif language == 'elixir' %}

Requires Elixir 1.14+. Add to your `mix.exs`:

```elixir
def deps do
  [
    {:html_to_markdown, "~> {{ version }}"}
  ]
end
```

{% elif language == 'r' %}

Requires R 4.3+ and a Rust toolchain (cargo, rustc).

```r
install.packages("htmltomarkdown", repos = "https://xberg-io.r-universe.dev")
```

Or install the development version from GitHub:

```r
devtools::install_github("xberg-io/html-to-markdown", subdir = "packages/r")
```

{% endif %}
