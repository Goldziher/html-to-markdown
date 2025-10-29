# Changelog

All notable changes to html-to-markdown will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.5.4] - 2025-10-30

### Changed
- Polished the Ruby gem messaging and README with performance highlights, configuration examples, and CLI guidance to match other language docs.
- Bumped every package (Rust crates, npm, PyPI, Ruby gem) to version 2.5.4.

## [2.5.3] - 2025-10-30

### Changed
- Publish Ruby gems as precompiled artifacts for Linux (x86_64), macOS (arm64 & x86_64), and Windows (x64) via a matrix GitHub Action, ensuring the CLI executable matches the target platform.
- Split the release workflow into prepare/build/publish stages so dry runs build artifacts without pushing, and trusted publishing now uploads every generated `.gem`.
- Hardened the gem preparation script to clear stale CLI binaries before copying in the platform-specific build output.
- Re-enabled the cross-language release workflow so crates.io, PyPI wheels/sdist, and both npm packages ship alongside the Ruby release.

## [2.5.2] - 2025-10-29

- Fix Ruby gem packaging to embed standalone Cargo manifest (no workspace inheritance) so installs compile out of tree successfully.
- Bump versions across Rust, Node, Python, and Ruby bindings.

## [2.5.1] - 2025-10-28

### Added
- Magnus-based Ruby gem (`html-to-markdown-rb`) with CLI proxy and comprehensive specs.

### Changed
- CI now includes Ruby coverage across macOS, Linux, and Windows, installing the appropriate toolchains (MSYS2 on Windows) for Magnus builds.
- Release workflow prepares the Ruby gem via trusted publishing alongside existing crates/npm packages.

### Fixed
- Bundler version pinned to 2.5.12 to support Ruby 3.2 CI environments.

## [2.5.0] - 2025-10-24

### Added

- **New `preserve_tags` option** - Preserve specific HTML tags in their original HTML form instead of converting them to Markdown. This is useful for complex elements like tables that may not convert well to Markdown. Fixes issue #95.
    - Accepts a list of tag names (e.g., `["table", "form"]`)
    - Preserves all attributes and nested content as HTML
    - Works independently of `strip_tags` - can use both options together
    - Available in all bindings: Rust, Python, Node.js, and WASM
    - Comprehensive test coverage in Rust, Python (pytest), and TypeScript (vitest)

### Changed

- **HTML preprocessing is now enabled by default** - The `PreprocessingOptions.enabled` default changed from `False` to `True` to ensure robust handling of malformed HTML. Users who want minimal preprocessing can explicitly set `enabled=False`.

### Fixed

- **Task list checkbox support** - Fixed sanitizer removing `<input type="checkbox">` elements when `remove_forms` is enabled (default). Checkboxes are now preserved during preprocessing to enable proper task list conversion (`- [x]` / `- [ ]`).
    - Added `input` tag to allowed tags in all sanitization presets (minimal, standard, aggressive)
    - Preserved `type` and `checked` attributes on input elements
    - Fixed pre-existing bug where task list checkboxes were silently removed
- **Data URI support for inline images** - Fixed sanitizer stripping `data:` URLs from image src attributes. Base64-encoded inline images (data URIs) are now preserved during preprocessing.
    - Added `data` to allowed URL schemes in all sanitization presets
    - Fixes `convert_with_inline_images` functionality for base64-encoded images
- **CDATA section handling** - Fixed test expectation for CDATA sections. CDATA sections are now correctly preserved as-is during HTML parsing instead of being partially stripped.
- **hOCR word spacing** - Fixed missing whitespace between `<span class="ocrx_word">` elements in hOCR documents. Words now have proper spaces between them.
    - Modified `OcrxWord` converter to insert space before each word if output doesn't end with whitespace or markdown formatting characters
    - Ensures proper word separation in OCR-generated documents without breaking markdown formatting (e.g., `*text*`, `[alt](url)`, `` `code` ``)
- **hOCR detection with preprocessing** - Fixed hOCR documents not being detected when HTML preprocessing is enabled (new default). The sanitizer now preserves:
    - `class` attributes on all elements (required for detecting hOCR element types)
    - `<meta>` tags with `name` and `content` attributes (required for hOCR metadata detection)
    - `<head>` tags (container for meta tags)
- **hOCR metadata extraction after sanitization** - Fixed metadata extraction failing when preprocessing strips the `<head>` container element. The extractor now finds orphaned meta tags anywhere in the document, not just inside `<head>` elements.
- **`preserve_tags` functionality with preprocessing** - Fixed `preserve_tags` not working when HTML preprocessing is enabled (the new default). The sanitizer now:
    - Accepts the `preserve_tags` list and allows those tags through sanitization
    - Preserves common HTML attributes (`id`, `class`, `style`, `title`, etc.) on preserved tags
    - Prevents `remove_forms` from stripping form tags when they're in the preserve list
    - Ensures tags and attributes survive preprocessing so they can be output as HTML
- **SVG support for inline image extraction** - Fixed SVG elements being stripped by the sanitizer, breaking inline image capture. All sanitization presets now allow:
    - SVG elements: `svg`, `circle`, `rect`, `path`, `line`, `polyline`, `polygon`, `ellipse`, `g`
    - SVG attributes: `width`, `height`, `viewBox`, `cx`, `cy`, `r`, `x`, `y`, `d`, `fill`, `stroke`
    - Enables `convert_with_inline_images` to capture inline SVG elements
- **Robust handling of malformed angle brackets in HTML** - Fixed parser failures when bare `<` or `>` characters appear in HTML text content (e.g., `1<2`, mathematical comparisons). The converter now:
    - Automatically escapes malformed angle brackets that aren't part of valid HTML tags
    - Works correctly with preprocessing both enabled and disabled
    - Handles edge cases like `1<2`, `1 < 2 < 3`, and angle brackets at tag boundaries
    - Fixes issue #94 where content following malformed angle brackets was lost
- Added comprehensive test coverage for malformed angle bracket handling in both Rust and Python test suites
- Fixed WASM build configuration to use correct `getrandom` backend for wasm32-unknown-unknown targets

## [2.4.1] - 2025-10-22

### Fixed

- Ensure npm publishes include the generated Node bindings and platform binaries by running the N-API build during CI.
- Configure WebAssembly builds with the `wasm_js` backend and strip wasm-pack `.gitignore` files so published packages ship the compiled `.wasm` artifacts.

## [2.4.0] - 2025-10-22

### Changed

- Updated Rust workspace dependencies (including `pyo3`) to their latest compatible releases and refreshed lockfiles.
- Normalized hOCR conversion spacing by collapsing stray triple newlines, ensuring generated Markdown matches regression fixtures.

### Fixed

- Corrected the WASM crate to depend on `getrandom`'s `wasm_js` feature, restoring WebAssembly builds.
- Expanded the Node package `files` list so published tarballs now include compiled `.node` artifacts, CommonJS shims, and typings.

## [2.3.4] - 2025-10-12

### Changed

- Incremented all distribution metadata and CLI version checks to 2.3.4 following the previous release tag conflict.
- Regenerated package metadata artifacts for the new patch release.

## [2.3.3] - 2025-10-12

### Added

- Python API now exports inline image helpers (`InlineImage`, `InlineImageWarning`, and `InlineImageConfig`) alongside `convert_with_inline_images`, with dedicated regression tests.
- Node and WASM bindings include inline image extraction examples and TypeScript definitions, validated by Vitest coverage.

### Changed

- Bumped all package metadata (Python, Rust, Node, WASM, CLI) to version 2.3.3 for a synchronized release.

### Fixed

- CLI `--version` test updated to assert the new release number.

## [2.2.0] - 2025-10-11

### Added

- `hocr_spatial_tables` option on `ConversionOptions` (Rust, Python, CLI) with `--no-hocr-spatial-tables` flag to disable spatial table reconstruction when desired.
- New hOCR regression fixtures for complex tables and code blocks to guard against formatting regressions.

### Changed

- Improved hOCR conversion heuristics to distinguish between dense paragraph layouts and true tables, yielding cleaner Markdown for scientific data.
- hOCR code-block detection now preserves fenced formatting, restoring context headings when present.

### Fixed

- CLI `--version` output and package metadata now report version 2.2.0 consistently.

## [2.1.1] - 2025-10-11

### Fixed

- Improve hOCR table reconstruction when tables are represented as paragraphs, ensuring Markdown tables are emitted for Tesseract outputs without explicit `ocr_table` markers.

## [2.1.0] - 2025-10-11

### Added

- **Inline image extraction** - New `convert_with_inline_images()` function to extract embedded images during conversion
    - Supports data URI images (`data:image/*`)
    - Supports inline SVG elements
    - Configurable via `InlineImageConfig` with options for:
        - Maximum decoded size limits
        - Custom filename prefixes
        - SVG capture control
        - Optional dimension inference for raster images
    - Returns `HtmlExtraction` with markdown, extracted images, and warnings
    - Available through both Rust and Python APIs

### Changed

- **Simplified API** - Removed `ParsingOptions` class in favor of direct `encoding` parameter on `ConversionOptions`
- **Automatic hOCR table extraction** - hOCR tables are now extracted automatically without requiring configuration
    - Removed `hocr_extract_tables` option (always enabled for hOCR content)
    - Removed `hocr_table_column_threshold` option (uses built-in heuristics)
    - Removed `hocr_table_row_threshold_ratio` option (uses built-in heuristics)
- Updated pre-commit hook versions (commitlint v9.23.0, pyproject-fmt v2.10.0, ruff v0.14.0)

### Fixed

- **hOCR metadata now uses YAML frontmatter** instead of HTML comments for cleaner markdown output
- **hOCR code organization** - Restructured spatial table reconstruction into dedicated `hocr/spatial.rs` module
- **Conservative table detection** - hOCR spatial table reconstruction now only applies to explicit `ocr_table` elements, preventing false positives
- Windows CLI binary detection - now correctly searches for `.exe` extension on Windows
- CLI binary bundling in Python wheels - binary now included in package for all platforms
- hOCR extractor Rust doctest - added missing import statement
- 928 Python test expectations updated for CommonMark-compliant v2 defaults
- Python 3.14-dev → Python 3.14 stable in CI workflows
- Reorganized wheel preparation script to `scripts/` directory
- Removed duplicate markdown documentation files (BENCHMARKS.md, PERFORMANCE.md, BENCHMARK_RESULTS.md, COMMONMARK_COMPLIANCE.md, REFACTORING_SUMMARY.md)

## [2.0.0] - 2025-10-03

### 🚀 Major Rewrite: Rust Backend

Version 2.0.0 represents a complete rewrite of html-to-markdown with a high-performance Rust backend, delivering **10-30x performance improvements** while maintaining full backward compatibility through a v1 compatibility layer.

### ⚠️ Breaking Changes

#### CommonMark-Compliant Defaults

V2 adopts CommonMark-compliant defaults for better interoperability:

| Option                  | V1 Default   | V2 Default | Reason                           |
| ----------------------- | ------------ | ---------- | -------------------------------- |
| `list_indent_width`     | 4            | 2          | CommonMark standard              |
| `bullets`               | "-"          | "\*+-"     | Cycling bullets for nested lists |
| `escape_asterisks`      | true         | false      | Minimal escaping                 |
| `escape_underscores`    | true         | false      | Minimal escaping                 |
| `escape_misc`           | true         | false      | Minimal escaping                 |
| `newline_style`         | "backslash"  | "spaces"   | CommonMark two-space line breaks |
| `code_block_style`      | "backticks"  | "indented" | CommonMark 4-space indent        |
| `heading_style`         | "underlined" | "atx"      | CommonMark `#` headings          |
| `preprocessing.enabled` | false        | false      | No change (opt-in)               |

**Migration**: If you relied on v1 defaults, explicitly set options to match v1 behavior.

#### Removed CLI Flags

The following v1 CLI flags are **not supported** in v2. The Python CLI proxy will raise helpful error messages when these flags are used:

| Removed Flag | Reason                | Migration                                 |
| ------------ | --------------------- | ----------------------------------------- |
| `--strip`    | Feature removed in v2 | Remove flag (feature no longer available) |
| `--convert`  | Feature removed in v2 | Remove flag (feature no longer available) |

**Note on Redundant Flags**: The following v1 flags are redundant in v2 (they match the defaults) but are **silently accepted** for backward compatibility:

- `--no-escape-asterisks`, `--no-escape-underscores`, `--no-escape-misc` (v2 defaults to minimal escaping)
- `--no-wrap` (v2 defaults to no wrapping)
- `--no-autolinks` (Rust CLI defaults to no autolinks)
- `--no-extract-metadata` (Rust CLI defaults to no metadata extraction)

These flags can be safely removed from your commands, or you can leave them for compatibility.

**Note**: The Rust CLI only supports positive flags (e.g., `--escape-asterisks`, `--autolinks`, `--wrap`). Negative flags (`--no-*`) are only supported through the Python CLI proxy for v1 compatibility.

#### CommonMark-Compliant List Formatting

- **Tight lists no longer have blank lines before nested sublists** - This follows the [CommonMark specification](https://spec.commonmark.org/) for list formatting
- Previous behavior (v1): `* Item 1\n\n    + Nested\n`
- New behavior (v2): `* Item 1\n    + Nested\n`
- **Why**: CommonMark specifies that tight lists (lists without blank lines between items) should not have blank lines before nested sublists
- **Impact**: Generated markdown will render identically in CommonMark-compliant renderers but may look different in source form
- **Migration**: If you need the old behavior for specific platforms, you can post-process the output or use loose lists (with blank lines between items)

### Added

#### Core Rust Implementation

- **Complete Rust rewrite** of HTML-to-Markdown conversion engine using `scraper` and `html5ever`
- **Native Rust CLI** with improved argument parsing and validation
- **PyO3 Python bindings** for seamless Rust/Python integration
- **Automatic hOCR table extraction** with built-in heuristics for OCR documents

#### New V2 API

- Clean, modern API with dataclass-based configuration
- `convert(html, options, preprocessing)` - primary API entry point
- `ConversionOptions` - comprehensive conversion settings (now includes `encoding`)
- `PreprocessingOptions` - HTML cleaning configuration
- Legacy parsing options removed in favour of explicit encoding on `ConversionOptions`
- Improved type safety with full type stubs (`.pyi` files)

#### V1 Compatibility Layer

- **100% backward compatible** v1 API through compatibility layer
- `convert_to_markdown()` function with all v1 kwargs
- Smart translation of v1 options to v2 dataclasses
- CLI argument translation for v1 flags
- Clear error messages for unsupported v1 features

#### Testing & Quality

- **77 new tests** for v1 compatibility (32 bindings + 26 CLI + 19 integration)
- Comprehensive integration tests with actual CLI execution
- Wheel testing workflow for cross-platform validation
- Python 3.10, 3.12, 3.14-dev test matrix
- Dual coverage reporting (Python + Rust)

#### CI/CD Improvements

- Shared build-wheels action for consistent wheel building
- Test-wheels workflow with full test suite on built wheels
- Rust coverage with `cargo-llvm-cov`
- Python coverage in LCOV format
- Automated wheel building for Python 3.10-3.13

### Changed

#### Performance

- **60-80x faster** than v1 for most conversion operations (144-208 MB/s throughput)
- Memory-efficient processing with Rust's zero-cost abstractions
- Optimized table handling with rowspan/colspan tracking
- Faster list processing with unified helpers

#### Architecture

- Removed Python implementation (`converters.py`, `processing.py`, `preprocessor.py`)
- Migrated to Rust-based conversion engine
- Simplified Python layer to thin wrapper around Rust bindings
- CLI now proxies to native Rust binary with argument translation

#### API Design

- More explicit configuration with separate option classes
- Better separation of concerns (conversion/preprocessing/parsing)
- Clearer parameter naming and organization
- Improved error messages and exception handling

### Removed v1 Features

The following v1 features were **removed** in v2:

- `code_language_callback` - Removed (use `code_language` option for default language)
- `strip` option - Removed (use preprocessing options instead)
- `convert` option - Removed (all supported tags are converted by default)
- `convert_to_markdown_stream()` - Removed (html5ever does not support streaming parsing)

### Not Yet Implemented

- `custom_converters` - Planned for future release with Rust and Python callback support

### Migration Guide

#### For Most Users (No Changes Needed)

If you're using the v1 API, your code will continue to work:

```python
from html_to_markdown import convert_to_markdown

# This still works in v2!
markdown = convert_to_markdown(html, heading_style="atx")
```

#### To Use New V2 API (Recommended)

```python
from html_to_markdown import convert, ConversionOptions

options = ConversionOptions(heading_style="atx")
markdown = convert(html, options)
```

#### CLI Changes

V1 CLI flags are automatically translated to v2:

```bash
# V1 style (still works)
html-to-markdown --preprocess-html --escape-asterisks input.html

# V2 style (recommended)
html-to-markdown --preprocess input.html  # escaping is default
```

### Performance Benchmarks

Real-world performance improvements over v1 (Apple M4):

| Document Type       | Size  | V2 Latency | V2 Throughput | Speedup vs V1 (2.5 MB/s) |
| ------------------- | ----- | ---------- | ------------- | ------------------------ |
| Lists (Timeline)    | 129KB | 0.62ms     | 208 MB/s      | **83x**                  |
| Tables (Countries)  | 360KB | 2.02ms     | 178 MB/s      | **71x**                  |
| Mixed (Python wiki) | 656KB | 4.56ms     | 144 MB/s      | **58x**                  |

V2's Rust engine delivers **60-80x higher throughput** than V1's Python/BeautifulSoup implementation across real-world documents.

### Technical Details

#### Rust Crates Structure

```text
crates/
├── html-to-markdown/       # Core conversion library
├── html-to-markdown-py/    # Python bindings (PyO3)
└── html-to-markdown-cli/   # Native CLI binary
```

#### Python Package Structure

```text
html_to_markdown/
├── api.py                  # V2 API
├── options.py              # V2 configuration dataclasses
├── v1_compat.py           # V1 compatibility layer
├── cli_proxy.py           # CLI argument translation
├── _rust.pyi              # Rust binding type stubs
└── __init__.py            # Public API exports
```

### Breaking Changes Summary

None if using v1 compatibility layer. If migrating to v2 API:

1. **Import changes**: `convert_to_markdown` → `convert`
1. **Configuration**: Kwargs → Dataclasses (`ConversionOptions`)
1. **Defaults changed**: See CommonMark-compliant defaults table above
1. **Removed features**: See [Removed v1 Features](#removed-v1-features) section above

### Complete V1 vs V2 Comparison

#### API Differences

| Aspect                  | V1                              | V2                                               |
| ----------------------- | ------------------------------- | ------------------------------------------------ |
| **Primary API**         | `convert_to_markdown(**kwargs)` | `convert(html, options, preprocessing, parsing)` |
| **Configuration**       | Keyword arguments               | Dataclasses (`ConversionOptions`, etc.)          |
| **Type Safety**         | Basic type hints                | Full `.pyi` stubs + generics                     |
| **Compatibility Layer** | N/A                             | `convert_to_markdown()` with v1 kwargs           |

#### Performance Differences

| Document Type       | V1 Throughput | V2 Throughput | Speedup |
| ------------------- | ------------- | ------------- | ------- |
| Lists (Timeline)    | 2.5 MB/s      | 208 MB/s      | **83x** |
| Tables (Countries)  | 2.5 MB/s      | 178 MB/s      | **71x** |
| Mixed (Python wiki) | 2.5 MB/s      | 144 MB/s      | **58x** |
| Average             | 2.5 MB/s      | 177 MB/s      | **71x** |

#### Implementation Differences

| Component        | V1                         | V2                       |
| ---------------- | -------------------------- | ------------------------ |
| **HTML Parser**  | BeautifulSoup4 / lxml      | html5ever (Rust)         |
| **Sanitizer**    | Custom Python              | ammonia (Rust)           |
| **Conversion**   | Pure Python (~3,850 lines) | Pure Rust (~4,800 lines) |
| **Bindings**     | N/A                        | PyO3                     |
| **CLI**          | Python wrapper             | Native Rust binary       |
| **Dependencies** | bs4, lxml, soupsieve       | None (statically linked) |

#### Output Differences (Default Settings)

| HTML                     | V1 Output                | V2 Output           |
| ------------------------ | ------------------------ | ------------------- |
| `<ul><li>Item</li></ul>` | `*   Item` (4 spaces)    | `- Item` (2 spaces) |
| `<h1>Title</h1>`         | `Title\n=====`           | `# Title`           |
| `Text*with*stars`        | `Text\*with\*stars`      | `Text*with*stars`   |
| `<br>`                   | Two trailing spaces      | Backslash `\`       |
| `<pre>code</pre>`        | ```` ```\ncode\n``` ```` | Indented 4 spaces   |

These differences reflect v2's alignment with CommonMark specification.

### Removed Python Implementation

- Python implementation of HTML conversion
- `html_to_markdown/converters.py` (1220 lines)
- `html_to_markdown/processing.py` (1195 lines)
- `html_to_markdown/preprocessor.py` (404 lines)
- `html_to_markdown/whitespace.py` (293 lines)
- `html_to_markdown/utils.py` (37 lines)
- Several test files migrated to Rust or marked as `.skip`

Total: **~3,850 lines** of Python code removed, replaced by **~4,800 lines** of Rust

### Notes

- **Platform Support**: Wheels built for Linux, macOS, Windows on x86_64
- **Python Version**: Requires Python 3.10+
- **ABI Compatibility**: Uses `abi3` for Python 3.10+ wheel reuse
- **Rust Version**: Built with stable Rust (tested on 1.75+)

______________________________________________________________________

## [1.x] - Previous Versions

For changes in v1.x releases, see git history before the v2 rewrite.

[2.0.0]: https://github.com/Goldziher/html-to-markdown/compare/v1.x...v2.0.0
