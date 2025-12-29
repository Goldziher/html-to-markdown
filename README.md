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


<img width="3384" height="573" alt="Linkedin- Banner" src="https://github.com/user-attachments/assets/1bd52e37-c45d-4f5c-8408-ee12997f6cfd" />

 
<div align="center" style="margin-top: 20px;">
  <a href="https://discord.gg/pXxagNK2zN">
      <img height="22" src="https://img.shields.io/badge/Discord-Join%20our%20community-7289da?logo=discord&logoColor=white" alt="Discord">
  </a>
</div>

High-performance HTML → Markdown conversion powered by Rust. Shipping as a Rust crate, Python package, PHP extension, Ruby gem, Elixir Rustler NIF, Node.js bindings, WebAssembly, and standalone CLI with identical rendering behavior across all runtimes.

## Key Features

- **Blazing Fast** – Rust-powered core delivers 10-80× faster conversion than pure Python alternatives (150–280 MB/s)
- **Polyglot** – Native bindings for Rust, Python, TypeScript/Node.js, Ruby, PHP, Go, Java, C#, and Elixir
- **Smart Conversion** – Handles complex documents including nested tables, code blocks, task lists, and hOCR OCR output
- **Metadata Extraction** – Extract document metadata (title, description, headers, links, images, structured data) alongside conversion
- **Visitor Pattern** – Custom callbacks for domain-specific dialects, content filtering, URL rewriting, accessibility validation
- **Highly Configurable** – Control heading styles, code block fences, list formatting, whitespace handling, and HTML sanitization
- **Tag Preservation** – Keep specific HTML tags unconverted when markdown isn't expressive enough
- **Secure by Default** – Built-in HTML sanitization prevents malicious content
- **Consistent Output** – Identical markdown rendering across all language bindings

**[Try the Live Demo →](https://kreuzberg-dev.github.io/html-to-markdown/)**

## Installation

Each language binding provides comprehensive documentation with installation instructions, examples, and best practices. Choose your platform to get started:

**Scripting Languages:**
- **[Python](./packages/python/README.md)** – PyPI package, metadata extraction, visitor pattern, CLI included
- **[Ruby](./packages/ruby/README.md)** – RubyGems package, RBS type definitions, Steep checking
- **[PHP](./packages/php/README.md)** – Composer package + PIE extension, PHP 8.2+, PHPStan level 9
- **[Elixir](./packages/elixir/README.md)** – Hex package, Rustler NIF bindings, Elixir 1.19+

**JavaScript/TypeScript:**
- **[Node.js / TypeScript](./packages/typescript/README.md)** – Native NAPI-RS bindings for Node.js/Bun, fastest performance, WebAssembly for browsers/Deno

**Compiled Languages:**
- **[Go](./packages/go/v2/README.md)** – Go module with FFI bindings, automatic library download
- **[Java](./packages/java/README.md)** – Maven Central, Panama Foreign Function & Memory API, Java 24+
- **[C#](./packages/csharp/README.md)** – NuGet package, .NET 8.0+, P/Invoke FFI bindings

**Native:**
- **[Rust](./crates/html-to-markdown/README.md)** – Core library, flexible feature flags, zero-copy APIs

**Command-Line:**
- **[CLI](https://crates.io/crates/html-to-markdown-cli)** – Cross-platform binary via `cargo install html-to-markdown-cli` or [Homebrew](https://formulae.brew.sh/formula/html-to-markdown)

<details>
<summary><strong>Metadata Extraction</strong></summary>

Extract comprehensive metadata during conversion: title, description, headers, links, images, structured data (JSON-LD, Microdata, RDFa). Use cases: SEO extraction, table-of-contents generation, link validation, accessibility auditing, content migration.

**[Metadata Extraction Guide →](./examples/metadata-extraction/)**

</details>

<details>
<summary><strong>Visitor Pattern</strong></summary>

Customize HTML→Markdown conversion with callbacks for specific elements. Intercept links, images, headings, lists, and more. Use cases: domain-specific Markdown dialects (Obsidian, Notion), content filtering, URL rewriting, accessibility validation, analytics.

**[Visitor Pattern Guide →](./examples/visitor-pattern/)**

</details>

<details>
<summary><strong>Performance & Benchmarking</strong></summary>

Rust-powered core delivers 150–280 MB/s throughput (10-80× faster than pure Python alternatives). Includes benchmarking tools, memory profiling, streaming strategies, and optimization tips.

**[Performance Guide →](./examples/performance/)**

</details>

<details>
<summary><strong>Tag Preservation</strong></summary>

Keep specific HTML tags unconverted when Markdown isn't expressive enough. Useful for tables, SVG, custom elements, or when you need mixed HTML/Markdown output.

See language-specific documentation for `preserveTags` configuration.

</details>

<details>
<summary><strong>Secure by Default</strong></summary>

Built-in HTML sanitization prevents XSS attacks and malicious content. Powered by ammonia with safe defaults. Configurable via `sanitize` options.

</details>

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:

- Setting up the development environment
- Running tests locally (Rust 95%+ coverage, language bindings 80%+)
- Submitting pull requests
- Reporting issues

All contributions must follow code quality standards enforced via pre-commit hooks (prek).

## License

MIT License – see [LICENSE](LICENSE) for details. You can use html-to-markdown freely in both commercial and closed-source products with no obligations, no viral effects, and no licensing restrictions.
