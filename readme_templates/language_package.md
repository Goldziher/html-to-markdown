<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://cdn.jsdelivr.net/gh/xberg-io/assets@v1/banner/readme-banner-dark.svg">
    <img alt="Xberg" width="420" src="https://cdn.jsdelivr.net/gh/xberg-io/assets@v1/banner/readme-banner-light.svg">
  </picture>
</p>

# html-to-markdown

{% include 'partials/_badges.md' %}

{{ description }}

## What This Package Provides

- **Same renderer as every binding** — output matches Rust, Python, Node.js, Ruby, PHP, Go, Java, .NET, Elixir, R, Dart, Swift, Zig, C FFI, and WASM.
- **Structured conversion result** — Markdown plus metadata, links, headings, images, tables, and warnings where the binding exposes them.
- **Production defaults** — HTML is parsed with the Rust core, sanitized by default, and rendered without runtime-specific Markdown drift.
  {% if language == "typescript" %}
- **Node-first TypeScript API** — native NAPI package, typed options/results, async-friendly integration for Node.js and Bun.
  {% elif language == "python" %}
- **Python-native packaging** — wheels for supported desktop/server platforms with a small, typed Python surface.
  {% elif language == "ruby" %}
- **Ruby extension** — Magnus-backed native extension with idiomatic Ruby objects and no separate service process.
  {% elif language == "php" %}
- **PHP extension** — typed PHP 8.2+ API with stubs suitable for PHPStan and production Composer projects.
  {% elif language == "go" %}
- **Go module** — cgo-backed binding with the Rust renderer behind ordinary Go functions and errors.
  {% elif language == "java" %}
- **Java package** — Panama FFM binding for direct native calls without a sidecar process.
  {% elif language == "csharp" %}
- **.NET package** — P/Invoke binding with record-shaped results and nullable-aware types.
  {% elif language == "elixir" %}
- **BEAM package** — Rustler NIF binding that keeps conversion inside the VM boundary.
  {% elif language == "r" %}
- **R package** — extendr binding for scripts, notebooks, and data workflows that need Markdown plus extracted metadata.
  {% elif language == "kotlin_android" %}
- **Android AAR** — bundled JNI libraries for Android targets; server-side Kotlin should use the Java package.
  {% elif language == "swift" %}
- **SwiftPM package** — swift-bridge types for macOS and iOS codebases.
  {% elif language == "dart" %}
- **Dart package** — flutter_rust_bridge API for Dart and Flutter projects.
  {% elif language == "zig" %}
- **Zig package** — thin wrapper over the C FFI with explicit allocation and error handling.
  {% endif %}

## Installation

{% include 'partials/_installation.md' %}

{% if migration_guide %}
{{ migration_guide }}
{% endif %}

{# Guarded on `benchmarks`, not on `performance` itself: swift, dart, zig, kotlin_android and wasm
   declare `[crates.readme.languages.<lang>.performance]` (platform/function/note) but no
   `[[...performance.benchmarks]]` rows, so the table rendered empty under a live heading. ~keep #}
{% if performance and performance.benchmarks %}

## Performance Snapshot

{{ performance | render_performance_table(name) }}

{% endif %}

## Coding-agent plugin

Install the [html-to-markdown plugin](https://github.com/xberg-io/html-to-markdown/blob/main/plugin/README.md)
for product skills and runtime integrations, or install [public skills](https://github.com/xberg-io/html-to-markdown/tree/main/skills)
independently:

```sh
npx skills add xberg-io/html-to-markdown
```

Select the skills and target agents interactively. Install the product CLI separately for
executable operations; the skill installer adds instructions only.

## Quick Start

{% include 'partials/_quick_start.md' %}

{% include 'partials/_architecture.md' %}

## API Reference

{% include 'partials/_api_reference.md' %}

{% include 'partials/_djot_output.md' %}

{% include 'partials/_plain_text_output.md' %}

{% if features.metadata_extraction %}

## Metadata Extraction

{% include 'partials/_metadata_extraction.md' %}
{% endif %}

{% if features.visitor_pattern %}

## Visitor Pattern

{% include 'partials/_visitor_pattern.md' %}
{% endif %}

{# No `## Examples` heading here: nothing has ever been rendered under it, so all 14 generated
   READMEs carried a dangling TOC entry. The per-language `[crates.readme.languages.<lang>].snippets`
   already feed Quick Start, Metadata Extraction and Visitor Pattern above. ~keep #}
{% include 'partials/_footer.md' %}
