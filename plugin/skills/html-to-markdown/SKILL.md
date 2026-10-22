---
description: Convert HTML to Markdown, Djot, or plain text with structured extraction. Use when writing code that calls html-to-markdown APIs in Rust, Python, TypeScript, Go, Ruby, PHP, Java, C#, Elixir, R, C, or WASM. Covers installation, conversion, configuration, metadata extraction, tables, document structure, inline images, URL fetching, and CLI usage.
license: MIT
metadata:
    author: xberg-io
    version: "0.1.0"
    repository: https://github.com/xberg-io/html-to-markdown
name: html-to-markdown
---

<!--
AI-RULEZ :: GENERATED FILE — DO NOT EDIT
Content-Hash: blake3:5abd004690ce3046e7a1c602d30fe8a23b961e3b6767988ebf6cc144c55d4a45
Source-Hash: blake3:4237e14c4d547597faf915235dde5b5d71e8c9d84681d23919d03302b738a0b4
Schema-Version: v1
-->

# html-to-markdown

html-to-markdown is a high-performance HTML→Markdown converter with a Rust core and 12 native language bindings. It converts HTML to CommonMark Markdown, Djot, or plain text in a single pass, optionally extracting metadata, tables, inline images, and a structured document tree.

Use this skill when writing code that:

- Converts HTML strings, files, or live URLs to Markdown, Djot, or plain text
- Extracts metadata (title, OG tags, JSON-LD/Microdata/RDFa, headers, links, images, language) from HTML
- Extracts structured table data (GFM markdown + cell grids) from HTML
- Extracts a structured document-structure tree
- Extracts inline images (data URIs, SVGs) from HTML
- Uses preprocessing to clean noisy HTML (ads, navigation, forms) before conversion

## Capability map

| Capability | CLI | SDKs |
| ---------- | --- | ---- |
| HTML→Markdown / Djot / plain text | `html-to-markdown FILE` | `convert(html, options)` |
| Read HTML from stdin / file / URL | `cat f \| …`, `FILE`, `--url URL` | `convert(htmlString, …)` |
| ~30 config options (headings, code blocks, lists, escaping, wrapping…) | flags | `ConversionOptions` |
| Metadata extraction | `--json` (default-extracted) | `result.metadata` |
| Table extraction | `--json --include-structure` → `tables[]` | `result.tables` with structure enabled |
| Document structure tree | `--json --include-structure` | `include_document_structure=true` |
| Inline image extraction | `--json --extract-inline-images` | `extract_images=true` |
| HTML preprocessing | `--preprocess [--preset …]` | `PreprocessingOptions` |
| Extraction-only (no Markdown body) | `--json --no-content` | options + read fields |

## Installation

### CLI

```bash
# (Homebrew 6.0+ requires explicit trust for third-party taps)
brew trust xberg-io/tap
brew install xberg-io/tap/html-to-markdown
# or download a prebuilt binary from the latest GitHub release:
#   https://github.com/xberg-io/html-to-markdown/releases/latest
# or build from source:
cargo install html-to-markdown-cli
```

### Language SDKs

```bash
pip install html-to-markdown                                # Python
pnpm add @xberg-io/html-to-markdown                      # TypeScript / Node.js
cargo add html-to-markdown-rs                                # Rust (features: metadata default; full = all)
gem install html-to-markdown                                 # Ruby
composer require xberg-io/html-to-markdown              # PHP
go get github.com/xberg-io/html-to-markdown/packages/go/v3   # Go
dotnet add package XbergIo.HtmlToMarkdown               # C#
pnpm add @xberg-io/html-to-markdown-wasm                 # WASM
```

- Java (Maven): `io.xberg:html-to-markdown`
- Elixir: `{:html_to_markdown, "~> 3.8"}` in `mix.exs`
- R: `install.packages("htmltomarkdown", repos = "https://xberg-io.r-universe.dev")`
- C (FFI): pre-built `.so` / `.dll` / `.dylib` from GitHub releases

## CLI vs SDK — which to use

- **CLI** — one-shot conversions, shell pipelines, fetching a single URL, ad-hoc metadata/table extraction via `--json | jq`. Flags only for conversion (`FILE` is positional; omit or use `-` for stdin); the only subcommand is `mcp`.
- **SDK** — embedding conversion in application code, batch processing, custom element conversion (visitor pattern, Rust), and tight loops where process spawn overhead matters.
- **MCP server** — `html-to-markdown mcp` exposes `convert_html` and `extract_metadata` as agent tools, so an MCP client can convert an HTML string directly with no shell-out. This plugin auto-registers it; see the **html-to-markdown-using-the-mcp-server** skill.

Both share the same `ConversionResult` shape, so output is interchangeable.

## When to use html-to-markdown vs xberg vs crawlberg

- **html-to-markdown** — you already have HTML (a string, a file, or a single URL) and want clean Markdown plus structured metadata/tables. No OCR, no document parsing, no crawling.
- **xberg** — you have *documents* (PDF, Office, images, email, archives) and need full text/table/metadata extraction with optional OCR. Use it when the input is not already HTML.
- **crawlberg** — you need to *crawl or scrape many pages*, follow links, and handle JS-rendered sites with a headless-Chrome fallback. It uses html-to-markdown internally for the HTML→Markdown step.

Rule of thumb: single HTML in → Markdown out = html-to-markdown. Many URLs / a site = crawlberg. Non-HTML documents = xberg.

## CLI quick start

```bash
# Convert a file to stdout
html-to-markdown input.html

# Convert and save
html-to-markdown input.html -o output.md

# Read from stdin
cat page.html | html-to-markdown

# Fetch and convert a URL
html-to-markdown --url https://example.com > out.md

# ConversionResult as JSON with structured tables
html-to-markdown --json --include-structure input.html

# JSON with document structure tree
html-to-markdown --json --include-structure input.html

# Extraction-only (no Markdown body)
html-to-markdown --json --no-content input.html

# Aggressive web-page cleanup
html-to-markdown input.html --preprocess --preset aggressive
```

## SDK quick start

### Rust

```rust
use html_to_markdown_rs::convert;

let result = convert("<h1>Hello World</h1><p>A paragraph.</p>", None)?;
println!("{}", result.content.unwrap_or_default());
```

### Python

```python
from html_to_markdown import convert

result = convert("<h1>Hello World</h1><p>A paragraph.</p>")
print(result.content)   # # Hello World\n\nA paragraph.
print(result.metadata)  # title, links, headers, …
```

### TypeScript / Node.js

```typescript
import { convert } from "@xberg-io/html-to-markdown";

// Node's convert() returns a ConversionResult object directly.
const result = convert("<h1>Hello World</h1><p>A paragraph.</p>");
console.log(result.content);
```

## ConversionResult fields

All languages return the same structure (dict, object, or struct).

| Field | Description |
| ----- | ----------- |
| `content` | Converted text (Markdown/Djot/plain). `null` only in extraction-only mode. |
| `metadata` | Title, OG, headers, links, images, structured data. |
| `tables` | Tables with `grid` (structured cells) and `markdown` fields. |
| `images` | Extracted inline images (requires inline-image extraction). |
| `document` | Structured document tree when structure extraction is enabled. |
| `warnings` | Non-fatal processing warnings (`message`, `kind`). |

## Configuration

All languages expose the same ~30 options. See [references/configuration.md](references/configuration.md) for the complete table. Common ones:

| Option | Values | Default |
| ------ | ------ | ------- |
| `heading_style` | `atx`, `underlined`, `atx-closed` | `atx` |
| `code_block_style` | `backticks`, `indented`, `tildes` | `backticks` |
| `output_format` | `markdown`, `djot`, `plain` | `markdown` |
| `wrap` / `wrap_width` | bool / 20–500 | off / `80` |
| `autolinks` (SDK) / `--no-autolinks` (CLI) | bool / flag | `true` (on); disable in CLI with `--no-autolinks` |
| preprocessing | `minimal` / `standard` / `aggressive` | off |

### Rust (builder)

```rust
use html_to_markdown_rs::{convert, ConversionOptions, HeadingStyle, OutputFormat};

let options = ConversionOptions::builder()
    .heading_style(HeadingStyle::Atx)
    .output_format(OutputFormat::Markdown)
    .wrap(true)
    .wrap_width(100)
    .build();
let result = convert(html, Some(options))?;
```

### Python (dataclass)

```python
from html_to_markdown import convert, ConversionOptions, PreprocessingOptions

html = "<h1>Title</h1><p>Body text.</p>"
result = convert(
    html,
    ConversionOptions(
        heading_style="atx",
        wrap=True,
        wrap_width=100,
        preprocessing=PreprocessingOptions(enabled=True, preset="aggressive"),
    ),
)
```

## Metadata extraction

The library `convert()` extracts metadata by default; the CLI needs `--json --extract-metadata` (see the **html-to-markdown-extracting-metadata** skill). Fields include `document` (title, description, language, canonical_url, open_graph), `headers`, `links` (with `link_type`), `images`, and `structured_data` (JSON-LD/Microdata/RDFa).

## Table extraction

With document structure extraction enabled, tables appear in `result.tables`, each with a pre-rendered `markdown` string and a structured cell `grid`. Markdown tables also appear inline in `content`. See the **html-to-markdown-extracting-tables** skill.

## Document structure extraction

Enable structure extraction (`--include-structure` on the CLI, `include_document_structure=true` in SDKs) to get a semantic node tree under `document`. Node types include `heading`, `paragraph`, `list`, `list_item`, `table`, `image`, `code`, `quote`, `group`, `metadata_block`.

## Common pitfalls

1. **`convert()` returns a result object, not a string.** Access `.content` for the Markdown text. This holds for Node.js too — `convert()` returns a `ConversionResult` object directly; do **not** `JSON.parse()` it.
2. **`--json` outputs JSON, not Markdown.** Omit `--json` for plain Markdown.
3. **`--include-structure`, `--extract-inline-images`, and `--no-content` require `--json`.**
4. **The conversion CLI is flags-only.** `FILE` is positional; the only subcommand is `mcp` (starts the MCP server).
5. **`--preset`, `--keep-navigation`, `--keep-forms` require `--preprocess`.**

## Additional resources

- **[CLI Reference](references/cli-reference.md)** — every flag, JSON shape, exit codes
- **[Configuration Reference](references/configuration.md)** — all 30+ options with defaults
- **[Rust API Reference](references/rust-api.md)** — signatures, builder, feature flags
- **[Python API Reference](references/python-api.md)** — functions, dataclasses, type hints
- **[TypeScript API Reference](references/typescript-api.md)** — functions, interfaces, Buffer support
- **[Other Bindings](references/other-bindings.md)** — Go, Ruby, PHP, Java, C#, Elixir, R, WASM, C FFI

GitHub: <https://github.com/xberg-io/html-to-markdown>
