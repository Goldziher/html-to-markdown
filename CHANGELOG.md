# Changelog

All notable changes to html-to-markdown will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- Node/WASM bundles now post-process their generated JS files to import the shared `WasmConversionOptions` typedef and emit typed doc comments (including typed inline-image `attributes`), so no `any` annotations leak into the published `dist`, `dist-node`, `dist-web`, or docs bundles.

## [2.11.3] - 2025-12-08
### Fixed
- Prevent link-label truncation from splitting multi-byte characters, which previously triggered a `PanicException` in the Python bindings when processing long anchors (resolves #139) and add a regression test to keep the truncation logic safe.

## [2.11.2] - 2025-12-07
### Added
- Explicitly ship typing artefacts in every binding: npm packages export `.d.ts` files by default, Ruby gems now include `sig/**/*.rbs` even when building outside git, and the Python wheel bundles `_html_to_markdown.pyi` plus a `py.typed` marker for static type checkers.

### Fixed
- Cleaned up the Python API’s inline-image helper to avoid redundant casts flagged by `mypy --strict`.
- Tightened PHP docblocks and psalm/phpstan annotations so option arrays use strongly typed shapes instead of `array<string, mixed>`.
- Hardened the WASM, Node, and Python bindings so their `options` argument is fully typed end-to-end (no `any` escapes in `.d.ts` files or placeholder `Any` annotations).

## [2.11.1] - 2025-12-05
### Fixed
- Preserve indentation in `<pre><code>` blocks while safely dedenting whitespace across multibyte characters to avoid panics when leading spaces are non-ASCII; regression fixture added for issue #134. Thanks @bbeardsley for the contribution.

## [2.11.0] - 2025-12-04
### Added
- CLI `--url` flag with optional `--user-agent` override to fetch remote HTML directly, plus charset-aware decoding.
- New GitHub Pages deploy workflow to publish the `docs/` demo from `main`.
- Additional CLI integration tests covering URL fetching (including custom UA, legacy markup, frameset/noframes, cp1252 decoding).

### Changed
- Demo layout now keeps input/output panes equal height and responsive.
- Rust core handles body-like content accidentally nested in `<head>` more gracefully.

## [2.10.1] - 2025-12-02

### Fixed
- Normalize whitespace inside link labels (collapse newlines and extra spaces) so anchors with messy HTML do not emit multi-line `[]` text.
- Flatten block children inside `<a>` (e.g., headings/paragraphs nested in anchors) into a single Markdown link instead of duplicating content; regression tests added for the reported Arabic product card case.

### Changed
- Synced all workspace/package versions to 2.10.1 via `task sync-versions`.

## [2.10.0] - 2025-12-02

### Added
- Centralized panic guarding for all bindings (Python, Node, PHP, WASM, C FFI) using a shared Rust helper so panics surface as language-native errors instead of unwinding across FFI boundaries.
- C FFI now stores the last error per thread and exposes it via `html_to_markdown_last_error`, with panic and UTF-8/null input diagnostics.
- Ruby binding now uses the shared panic guard and emits consistent panic messages; specs cover panic interception across conversion entrypoints.

### Changed
- Wasmtime test harness initializes conversion options via struct literals to reduce clippy noise in CI.

### Fixed
- Rust coverage CI now forces `cargo-llvm-cov` reinstall to avoid cached binary conflicts on GitHub runners.
- PHP smoke tests use the Packagist package name `goldziher/html-to-markdown`, matching README install instructions.

## [2.9.3] - 2025-12-01

### Changed
- **Version sync** – Bumped the entire workspace (Rust, Python, npm, Ruby, Elixir, Java, C#, Go) to 2.9.3 via `task sync-versions` to prep the next patch release.
- **Docs & install commands** – Pointed all Composer references to the published `goldziher/html-to-markdown` package and clarified npm usage to the shipped packages (`html-to-markdown-node` / `html-to-markdown-wasm`).

### Fixed
- **Go lint CI** – Replaced the invalid `go fmt -l` invocations with `gofmt -l` in the Taskfile so `task check`/CI lint runs complete successfully on Go 1.25.

## [2.9.2] - 2025-11-28

### Fixed
- **UTF-8 safety (Fix #127)** – Guarded whitespace trimming against mid-codepoint truncation, eliminating byte-boundary panics on multilingual documents; added fixture and regression test for the reported Ruby-path crash.
- **Image conversion (Fix #128)** – `<img>` elements with `width`/`height` now render as Markdown images instead of raw HTML; regression test covers inline-data URIs with dimensions.

## [2.9.1] - 2025-11-22

### Changed
- **HTML repair fallback** – Minified or malformed pages now reparse via html5ever when inline/block nesting is broken, keeping content that previously vanished (e.g., SPA shells and Hacker News markup).
- **Link label recovery** – Anchor text fallback prefers child formatting or hrefs only when appropriate, preventing empty labels while keeping CommonMark empty-link semantics intact.

### Fixed
- **Layout tables to lists** – Headless tables with mixed column counts/spans or nested tables render as list rows instead of broken Markdown tables, restoring Hacker News output.
- **Issue 121 regressions** – Added fixtures/tests for the empty SPA and malformed Hacker News samples; both now produce full Markdown content without frontmatter noise.

## [2.9.0] - 2025-11-20

### Added
- **Elixir bindings** – New `html_to_markdown` Hex package built with Rustler, exposing the Rust core converter to Elixir with configurable options plus `convert/2` and `convert!/2`.
- **WASM runtime verification** – Added a Wasmtime-backed e2e suite (`e2e/wasm-wasmtime`) plus `task wasm:test:wasmtime` to compile the `html-to-markdown-wasm` artefact for `wasm32-unknown-unknown` and execute it inside Wasmtime. CI now runs these tests to ensure the WASM package works outside the browser runtime.

### Changed
- **Astral `tl` parser** – The HTML parser dependency now points to the actively maintained `astral-tl` fork (still imported as `tl`) so comment parsing stays up to date with upstream fixes.
- **NuGet Package ID** – C# bindings now publish under `Goldziher.HtmlToMarkdown` to avoid clashing with an existing community package.
- **Wasmtime CI Coverage** – The Wasmtime e2e job now runs on Linux x64, Linux arm64, macOS, and Windows runners so every GitHub-hosted architecture executes the WASM tests.

### Fixed
- **PHP PIE source bundle** – Release packaging strips the Wasmtime e2e workspace from the staged `Cargo.toml`, fixing the “failed to load manifest” error in the publish workflow.
- **Horizontal rule rendering** – `<p>…</p><hr>` now emits a blank line before `---` while preserving blockquote spacing so the rule is never misinterpreted as a setext heading.
- **Empty HTML comments** – Zero-width `<!---->` comment nodes are normalized before parsing, so comment placeholders no longer cause the following content to disappear.

## [2.8.3] - 2025-11-15

### Changed
- **Deterministic uv installs** – Every `uv sync` invocation in CI and the Taskfile now runs with `--no-install-workspace`, ensuring Python dependencies are resolved without mutating editable installs before the subsequent build/test steps run.

### Fixed
- **NuGet Publishing** – Release automation now uses GitHub’s trusted publisher flow via `NuGet/login@v1` (OIDC → short-lived API key) before pushing artifacts, removing the dependency on long-lived secrets.
- **Hex Publishing** – The release workflow invokes `mix hex.publish --yes` from `packages/elixir`, with `ex_doc` bundled as a dev dependency so documentation generation works during release.

## [2.8.2] - 2025-11-15

### Changed
- **Unified Version Sync** – `scripts/sync_versions.py` now updates Elixir `@version` declarations, the C# `.csproj`, and the Java `pom.xml` (alongside every npm/pyproject/Gemfile manifest). `task sync-versions` bumps the entire multi-language stack to **2.8.2** in one shot.
- **CI / Release Toolchains** – GitHub Actions now installs Elixir dependencies ahead of Credo and runs on **Elixir 1.19 + OTP 28.1**, matching the README prerequisites and preventing per-job regex recompilation warnings.
- **Taskfile Coverage** – Added `elixir:update` plus full `java:{install,update,test,lint}` tasks so `task setup`, `task update`, `task test`, and `task lint` cover every published runtime (Go, C#, Elixir, Java) just like the CI workflows.

## [2.8.1] - 2025-11-15

### Fixed
- **Release Pipeline** – Bumped all package manifests to v2.8.1 so the publish workflow can push fresh artifacts after the v2.8.0 smoke-test fixes (PyPI, npm, and RubyGems refuse re-uploads of the same version).

## [2.8.0] - 2025-11-15

### Added
- **Java, C#, and Go Bindings (First Release)** – First public release of official Java (JNA), C# (.NET), and Go (CGO) language bindings. All three are integrated into the unified `task bench:bindings` harness and ship with comprehensive performance data in their READMEs. C# leads at ~1.4k ops/sec (≈171 MB/s), Go at ~1.3k ops/sec (≈165 MB/s), and Java at ~1.0k ops/sec (≈126 MB/s) on the 129 KB Wikipedia lists fixture.

### Changed
- **BREAKING: Preprocessing Disabled by Default** – HTML preprocessing is now disabled by default in the library API to prevent silent content loss. Previously, `<nav>`, `<form>`, and related elements (along with all their children) were dropped by default, causing important content inside these tags to be lost. Users who want preprocessing must now explicitly enable it via `PreprocessingOptions { enabled: true, ... }`. The CLI behavior is unchanged (preprocessing has always been opt-in with `--preprocess`).
- **Rust Toolchain Settings** – All crates (including the Ruby binding) now inherit `edition = "2024"` and `rust-version = "1.85"` from the workspace to keep toolchain configuration centralized.
- **GitHub Actions Workflow DRY** – Created 17 reusable composite actions (8 build actions + 9 smoke test actions) to eliminate ~267 lines of duplication between CI and publish workflows.
- **Toolchain Management** – Migrated to official GitHub Actions parameters for Ruby Bundler 2.7.2 and PHP Composer 2.9.1, removing manual installation scripts.

### Fixed
- **Windows PHP Extension Build** – Replaced php-windows-builder orchestration with direct `cargo build` matching ext-php-rs's proven approach, resolving LLVM 19 MMX header incompatibilities and Zend symbol linking errors.
- **Linux PHP Build** – Added php-config path capture and parameter passing to build-php-linux action, fixing "php-config executable not found" errors.
- **Ruby Linux Build** – Set LD_LIBRARY_PATH on Linux builds to match magnus best practices, preventing potential "strings.h not found" errors.
- **golangci-lint CI** – Split golangci-lint pre-commit hook into separate invocations for `packages/go` and `examples/go-smoke` modules, fixing "directory prefix does not contain main module" errors by running each check from within its Go module directory.
- **Windows Go CGO Smoke Test** – Documented MSVC/MinGW toolchain incompatibility and skip Windows Go smoke test with informative message, as Go CGO uses MinGW which cannot link against MSVC-compiled Rust FFI libraries.
- **Go Code Quality** – Removed redundant newline in `examples/go-smoke/main.go` fmt.Println call (detected by newly-working golangci-lint).

## [2.7.2] - 2025-11-12

### Fixed
- **Node/WASM Binding Regression** – HTML preprocessing no longer drops `<html>`, `<head>`, or `<body>` wrappers when their classes resemble navigation chrome, so large Wikipedia fixtures once again emit full markdown (restoring the Vitest length/table expectations for Node bindings and keeping WASM conversions consistent).
- **Cloudflare WASM Initialization** – Bundler builds of `html-to-markdown-wasm` now expose `initWasm()`/`wasmReady` so edge runtimes that instantiate WebAssembly modules asynchronously (Cloudflare Workers, Vite dev servers, etc.) can await initialization before calling `convert()`, eliminating the `__wbindgen_start` runtime error.
- **Footer Retention (Fix #120)** – The Rust preprocessor keeps plain `<footer>` content unless the element carries explicit navigation hints (role/class/id). Python and Rust conversions once again preserve footer copy while still stripping true navigation footers such as `.site-footer` menus.
- **Release Smoke Coverage** – The publish workflow now downloads the built artifacts (Node, WASM, Python wheels, Ruby gems, PHP zips) and reruns the README smoke installs across Linux/macOS/Windows before any packages are uploaded, ensuring we're testing the exact bits we ship.

## [2.7.1] - 2025-11-12

### Added
- **Language-Specific Benchmarks** – Every binding README (Node, WASM, Python, Ruby, PHP, TypeScript) now publishes the latest `task bench:bindings` throughput numbers so runtime documentation stays aligned with the shared fixtures.
- **Examples/Smoke Suite** – Added `examples/{node,wasm,python,ruby,php,rust}-smoke` plus an overview README to exercise both the published artifacts and local builds before a release.

### Changed
- **Docs Accuracy** – Node/WASM READMEs now clearly reference the real npm packages (`html-to-markdown-node`, `html-to-markdown-wasm`) and provide correct import samples.
- **TypeScript README** – Highlights that the CLI wrapper inherits the native Node benchmarks.
- **Repository Hygiene** – `.gitignore` now drops `.venv/`, vendor directories, and nested `node_modules/` so smoke tests and language-specific toolchains don’t dirty the tree.
- **Ruby Build Metadata** – `extconf.rb` uses a relative path for the embedded Cargo crate and the crate’s `Cargo.toml` now declares explicit `edition`, `rust-version`, and dependency pins, allowing `gem install` outside the workspace.
- **Version Sync Script** – `scripts/sync_versions.py` updates every `html-to-markdown-rs` dependency pin (workspace root plus downstream crates) to keep cross-language releases in lockstep.

### Fixed
- **Smoke Test Coverage** – Verified Node, WASM, Python, Ruby (local gem), PHP (Composer path repo), and Rust installs; documented gaps where external registries still need to publish `goldziher/html-to-markdown` or `html-to-markdown` 2.7.1 before release.

## [2.7.0] - 2025-11-12

### Added
- **Zero-Copy Inline Images** – Node/N-API and WASM bindings now expose `convertInlineImagesBuffer` / `convertBytesWithInlineImages`, letting benchmark harnesses feed `Buffer`/`Uint8Array` data directly without creating intermediate JS strings.

### Changed
- **Rust Core Preprocessing** – HTML normalization (self-closing fixes, malformed `<` escaping, script/style stripping) now happens in a single streaming pass that hands owned buffers straight to `tl::parse_owned`, cutting multiple allocations from every conversion.
- **Benchmark Harness + Docs** – Re-ran the cross-language runtime suite after the Rust core optimizations and refreshed the README tables, keeping the published throughput numbers (Node/Python/Rust/WASM/PHP) in sync with `tools/runtime-bench/results/latest.json`.
- **Version Alignment** – Bumped every package (Rust crates, npm packages, PyPI distribution, Ruby gem, PHP extension, WASM bundle) to `2.7.0` via `task sync-versions`.

### Fixed
- **Ruby Benchmark Output** – The Ruby benchmark driver now emits JSON without relying on `json` native extensions, preventing `libruby` incompatibility errors during `task bench:bindings`.
- **Nested `<strong>` Normalization (Fix #111)** – The Rust converter now tracks when bold markup is already active, so nested `<b>`/`<strong>` combinations (including `<mark>`, `<summary>`, `<legend>`) no longer generate `****` artifacts (`<b>bo<b>ld</b>er</b>` correctly becomes `**bolder**`). The CommonMark harness documents the four spec examples that expect stacked markers and skips them accordingly.
- **Heading Whitespace (Fix #118)** – ATX/Setext headings swallow layout-only newlines and indentation inside `<h1>…<h6>` so pretty-printed HTML like `<h2>Heading\n  Text</h2>` renders as a single Markdown heading line.
- **Inline Whitespace Preservation** – Reworked the inline text pipeline so removing zero-width inline elements (e.g., `<input>`, `<script>`, empty `<b>`) no longer collapses surrounding spaces; fixtures like `test_chomp`, `test_form_with_inputs_inline_mode`, and checkbox/task-list rendering now match their expected double-space gaps.
- **DOCTYPE Handling (Fix #119)** – `<!DOCTYPE …>` declarations are stripped during preprocessing so they never leak as stray `PUBLIC…` text in the output, even when metadata extraction is enabled.

## [2.6.6] - 2025-11-10

### Changed
- **Ruby Gem Packaging** – Moved the `html-to-markdown-rb` crate under `packages/ruby/ext/html-to-markdown-rb/native` and pointed `extconf.rb` at that path so every published gem now contains the Cargo sources it needs to compile on install.
- **Documentation Consistency** – Updated the root, crate, and package READMEs to drop references to the unrelated `html-to-markdown` npm package and to consistently list our supported targets (Node, WASM, Python, Ruby, PHP, CLI).
- **Dependency Refresh** – Ran `task update` to upgrade Rust crates, npm packages, Bundler gems, Python requirements, and Composer dependencies across the monorepo.

### Fixed
- **Rust Clippy Lints** – Addressed `clippy::unnecessary-map-or` in the converter and hOCR table builder by using `.is_none_or`, keeping inline-image filtering and column pruning logic clear while allowing `cargo clippy -D warnings` to pass.
- **PIE Source Packaging** – `scripts/package_php_pie_source.sh` now copies `packages/ruby/.../native` into the temporary workspace so the Ruby crate exists when PIE builds the PHP extension.

## [2.6.3] - 2025-11-07

### Fixed
- **Release Pipeline** - Fixed missing `is_tag` output in publish workflow that caused all publishing jobs to be skipped
- **Node.js Package Dependencies** - Added missing `optionalDependencies` to html-to-markdown-node package.json to properly link platform-specific binaries
- **Version Management** - Created centralized version sync script (`scripts/sync_versions.py`) to maintain consistency across all package manifests (Rust, Node.js, Python, Ruby, WASM)
- **Cargo Workspace** - Aligned html-to-markdown-rb crate version (was 2.5.7) with workspace version

### Changed
- Added `task sync-versions` command to Taskfile for easy version synchronization across the monorepo

## [2.6.2] - 2025-11-07

### Fixed
- **Table Rowspan Support** - Fixed tables with rowspan cells to correctly duplicate cell content across spanned rows instead of showing empty cells (fixes #116)
- **Node.js Platform Package Publishing** - Fixed workflow to correctly move packed .tgz files to npm directory for publishing
- **Deprecation Warnings** - Updated CLI tests to use `CARGO_BIN_EXE` env var instead of deprecated `cargo_bin` method
- **Deprecation Warnings** - Replaced deprecated `criterion::black_box` with `std::hint::black_box` in benchmarks
- **Clippy Warnings** - Fixed field assignment warnings by using struct initialization with defaults

## [2.6.1] - 2025-11-07

### Fixed
- **Node.js Platform Packages** - Fixed publishing of platform-specific npm packages. The workflow now correctly packs npm directories into .tgz files before publishing, ensuring all platform bindings (linux-x64-gnu, darwin-arm64, win32-x64-msvc, etc.) are published to npm.
- **WASM Package Publishing** - Added proper WASM package publishing workflow to ensure html-to-markdown-wasm is published to npm registry.

## [2.6.0] - 2025-11-07

### Added
- **PHP Extension Support** - Official PHP extension (`goldziher/html-to-markdown`) providing native HTML to Markdown conversion for PHP 8.2+
  - Built with ext-php-rs for high-performance Rust-backed conversion
  - Supports both Thread-Safe (TS) and Non-Thread-Safe (NTS) builds
  - Available for Windows (x86, x64), Linux, and macOS
  - Distributed via PIE (PHP Installer for Extensions) source bundles
  - Prebuilt Windows binaries for PHP 8.2, 8.3, and 8.4
  - Comprehensive test suite with PHPUnit

### Changed
- Refactored PHP build variable names from `HTM2MD_*` to `HTMLTOMARKDOWN_*` for improved clarity in Makefile.frag and config.m4
- Bumped all package versions to 2.6.0 across Rust crates, npm packages, PyPI wheels, Ruby gem, and PHP extension

## [2.5.7] - 2025-11-03

### Added
- Publish Windows PHP extension binaries alongside the PIE source bundle during the release pipeline, enabling one-click installs on every platform.
- Build and archive the CLI binary for Linux (gnu & musl), macOS arm64, and Windows x86_64, plus ship prebuilt WASM bundles (dist/dist-node/dist-web) so every runtime gets first-class artifacts.

### Changed
- Renamed the PHP extension package to `goldziher/html-to-markdown`, moved the Composer metadata to the repository root, and refreshed the documentation/badges for every language target.
- Bumped every package (Rust crates, npm packages, PyPI wheels, Ruby gem, PHP extension) to version 2.5.7.
- Restored the Node.js N-API build matrix so macOS, Windows, and Linux binaries ship automatically with each npm release.

### Fixed
- Preserve ordered list numbering and indentation when list items render headings or HTML tables, so mixed block content stays under the correct bullet (fixes #107).

## [2.5.6] - 2025-10-30

### Changed
- The Ruby gem now packages its own README at the gem root, so RubyGems renders the fully formatted documentation (benchmarks, configuration, CLI notes) without broken links.
- Documentation links: the Ruby README now surfaces GitHub resources (issues, changelog, live demo) alongside feature highlights.
- Bumped every package (Rust crates, npm, PyPI, Ruby gem) to version 2.5.6.

## [2.5.5] - 2025-10-30

### Changed
- Synced documentation: the root README now links to every language guide, and the Ruby README highlights GitHub resources alongside feature docs.
- Gem packaging now reads the README directly for the RubyGems long description while keeping Rubocop happy on all Ruby sources.
- Bumped every package (Rust crates, npm, PyPI, Ruby gem) to version 2.5.5.

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
| **Sanitizer**    | Custom Python              | html5ever DOM filtering  |
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
