# Language Bindings

The HTML-to-Markdown engine ships native packages for every runtime we support.
Each binding wraps the same Rust core, so feature parity and output consistency
are guaranteed across languages.

## Contents

- [Rust crate](#rust-crate-html-to-markdown-rs)
- [Node.js native addon](#nodejs-native-addon-html-to-markdown-node)
- [WebAssembly / universal JS](#webassembly--universal-js-html-to-markdown-wasm)
- [Python](#python-html-to-markdown)
- [Ruby](#ruby-html-to-markdown)
- [PHP](#php-extension--composer-package-goldziherhtml-to-markdown)
- [Elixir](#elixir-html_to_markdown)
- [Go](#go-githubcomgoldziherhtml-to-markdownpackagesgohtmltomarkdown)
- [Java](#java-iogithubgoldziherhtml-to-markdown)
- [.NET](#net-goldziherhtmltomarkdown)
- [Command-line](#command-line-html-to-markdown-cli)

---

## Rust crate (`html-to-markdown-rs`)

### Install

```bash
cargo add html-to-markdown-rs
```

### Usage

```rust
use html_to_markdown::convert;

fn main() {
    let html = "<h1>Hello</h1><p>Rust binding</p>";
    let markdown = convert(html, None);
    assert_eq!(markdown.trim(), "# Hello\n\nRust binding");
}
```

Enable `inline-images` or `attachments` features in `Cargo.toml` to mirror the
capabilities exposed in the higher-level bindings.

---

## Node.js native addon (`html-to-markdown-node`)

### Install

```bash
npm install html-to-markdown-node
# or: yarn add html-to-markdown-node
# or: pnpm add html-to-markdown-node
```

Prebuilt binaries are published for every major platform (`darwin`, `linux`
glibc/musl, `win32`, and `linux armv7`). npm resolves the correct binary package
automatically.

### Usage

```typescript
import { convert, convertWithInlineImages } from "html-to-markdown-node";

const markdown = convert("<h1>Hello</h1><p>from Node</p>", {
  headingStyle: "Atx",
  codeBlockStyle: "Backticks",
});

const extraction = convertWithInlineImages(
  '<img src="data:image/png;base64,..." alt="demo">',
  null,
  { inferDimensions: true },
);

console.log(markdown);
console.log(extraction.inlineImages.length);
```

The TypeScript definitions mirror the Rust configuration surface, including
inline image extraction warnings.

---

## WebAssembly / universal JS (`html-to-markdown-wasm`)

### Install

```bash
npm install html-to-markdown-wasm
```

Three entry points are published:

- `dist/` — bundler-friendly ESM (`type: module`)
- `dist-node/` — Node.js/Bun (CommonJS + ESM)
- `dist-web/` — browser-ready ES modules hosted on npm/CDNs

### Usage

**Bundler / Node.js**

```typescript
import { convert } from "html-to-markdown-wasm";

const md = convert("<h1>WASM</h1>", { headingStyle: "atx" });
```

**Browser**

```html
<script type="module">
  import init, { convert } from "https://unpkg.com/html-to-markdown-wasm/dist-web/html_to_markdown_wasm.js";
  await init();
  console.log(convert("<strong>Runs everywhere!</strong>"));
</script>
```

WebAssembly is ideal when you need portability across browsers, edge workers, or
JavaScript runtimes without native addon support. Prefer the Node.js addon for
maximum performance on server-side workloads.

---

## Python (`html-to-markdown`)

### Install

```bash
pip install html-to-markdown
```

Manylinux wheels ship for CPython 3.9–3.13 on Linux, macOS (universal2), and
Windows (x64 + ARM64).

### Usage

```python
from html_to_markdown import convert, convert_with_inline_images

markdown = convert("<h1>Hello</h1><p>Python binding</p>")

extraction = convert_with_inline_images(
    '<img src="data:image/png;base64,..." alt="example">',
    inline_infer_dimensions=True,
)

print(markdown)
print(len(extraction.inline_images))
```

---

## Ruby (`html-to-markdown`)

### Install

```bash
gem install html-to-markdown
```

Platform-specific gems are published for Linux (x64), macOS (x64/arm64), and
Windows (x64). Bundler resolves the appropriate gem automatically.

### Usage

```ruby
require "html/to_markdown"

Markdown.convert("<h1>Hello</h1><p>Ruby binding</p>").tap do |markdown|
  puts markdown
end
```

Inline image APIs, configuration objects, and error surfaces map one-to-one with
the Rust engine.

---

## PHP extension + Composer package (`goldziher/html-to-markdown`)

The PHP binding ships as a native extension distributed via
[PIE](https://github.com/php/pie) and a typed Composer package.

```bash
pie install goldziher/html-to-markdown --install-project
composer require goldziher/html-to-markdown
```

Once installed, enable the extension in `php.ini` and use the `HtmlToMarkdown`
service classes or procedural helpers. See [docs/php.md](php.md) for full
installation, distribution, and usage guidance (including Windows DLLs).

---

## Elixir (`html_to_markdown`)

### Install

```elixir
{:html_to_markdown, "~> 2.11"}
```

The package ships a Rustler NIF built from the same Rust core for consistent
conversion results.

### Usage

```elixir
{:ok, md} = HtmlToMarkdown.convert("<h1>Hello</h1><p>Elixir</p>")
IO.puts(md)
```

---

## Go (`github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown`)

### Install

```bash
go get github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown
```

The Go package links against the C FFI library built from the Rust core.

### Usage

```go
package main

import (
    "fmt"
    htm "github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown"
)

func main() {
    md, err := htm.Convert("<h1>Hello</h1><p>Go binding</p>", nil)
    if err != nil {
        panic(err)
    }
    fmt.Println(md)
}
```

---

## Java (`io.github.goldziher:html-to-markdown`)

### Install (Maven)

```xml
<dependency>
  <groupId>io.github.goldziher</groupId>
  <artifactId>html-to-markdown</artifactId>
  <version>2.11.1</version>
</dependency>
```

### Usage

```java
import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;

public class Demo {
    public static void main(String[] args) {
        var md = HtmlToMarkdown.convert("<h1>Hello</h1><p>Java binding</p>");
        System.out.println(md);
    }
}
```

---

## .NET (`Goldziher.HtmlToMarkdown`)

### Install

```bash
dotnet add package Goldziher.HtmlToMarkdown --version 2.11.1
```

### Usage

```csharp
using HtmlToMarkdown;

var md = Converter.Convert("<h1>Hello</h1><p>.NET binding</p>");
Console.WriteLine(md);
```

---

## Command-line (`html-to-markdown-cli`)

### Install

```bash
cargo install html-to-markdown-cli
```

### Usage

```bash
html-to-markdown < input.html > output.md
# or
html-to-markdown https://example.com > output.md
```

Use `--help` for a complete list of flags, output formatting options, and
profiling tools.
