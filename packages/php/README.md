# html-to-markdown PHP package

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown.svg)](https://crates.io/crates/html-to-markdown)
[![npm (node)](https://badge.fury.io/js/html-to-markdown-node.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://badge.fury.io/js/html-to-markdown-wasm.svg)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/Goldziher.HtmlToMarkdown.svg)](https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

Typed PHP bindings for the `html_to_markdown` native extension generated from
the Rust html-to-markdown engine. The API mirrors the behaviour exposed by the
Rust, Python, Ruby, Node.js, and WASM distributions while embracing modern PHP
features (readonly value objects, enums, typed exceptions).

## Requirements

- PHP 8.2 or newer
- `html_to_markdown` native extension (install via PIE `goldziher/html-to-markdown` or download the GitHub release artifacts)
- Composer for dependency management

## Platform Support

The native extension builds and runs on **Linux and macOS** (x86_64 and ARM64).

**Windows is not currently supported** due to limitations in the ext-php-rs library that prevent proper symbol resolution during the build process. Specifically, the Windows PHP SDK does not export certain Zend engine symbols (e.g., `zend_ce_exception`, `zend_ce_traversable`) in a way that the Rust linker can resolve them. This is a known limitation of ext-php-rs 0.15.x on Windows.

If you require Windows support, consider using one of the alternative distributions:
- **Node.js** (`html-to-markdown-node`)
- **Python** (`html-to-markdown`)
- **WebAssembly** (`html-to-markdown-wasm`)

## Installation

```bash
pie install goldziher/html-to-markdown        # Native extension via PIE
composer require goldziher/html-to-markdown   # Typed PHP API wrappers
```

If the extension is not enabled the post-install script prints the steps
required to install or activate it. Once enabled, ensure `extension=html_to_markdown`
is present in `php.ini`.

## Quick start

```php
use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Service\Converter;
use function HtmlToMarkdown\convert;
use function HtmlToMarkdown\convert_with_inline_images;

// Object-oriented usage
$converter = Converter::create();
$markdown = $converter->convert('<h1>Hello</h1>');

$extraction = $converter->convertWithInlineImages(
    '<img src="data:image/png;base64,Zm9v" alt="demo">',
    new ConversionOptions(),
    new InlineImageConfig(inferDimensions: true),
);

// Procedural helpers delegate to the converter
$markdown = convert('<p>Lorem ipsum</p>');
$extraction = convert_with_inline_images('<img src="data:image/png;base64,Zm9v" alt="demo">');
```

## Performance (Apple M4)

Captured via `task bench:bindings -- --language php` so they match the data in the root README:

| Document               | Size   | ops/sec (PHP) |
| ---------------------- | ------ | ------------- |
| Lists (Timeline)       | 129 KB | 533           |
| Tables (Countries)     | 360 KB | 118           |
| Medium (Python)        | 657 KB | 59            |
| Large (Rust)           | 567 KB | 65            |
| Small (Intro)          | 463 KB | 83            |
| hOCR German PDF        | 44 KB  | 1,007         |
| hOCR Invoice           | 4 KB   | 8,781         |
| hOCR Embedded Tables   | 37 KB  | 1,194         |

> Throughput sits in the tens of MB/s range once the extension is loaded; startup time is dominated by compiling the Rust library.

## Configuration

- `ConversionOptions` is an immutable value object covering all options exposed
  by the Rust converter (heading style, wrapping, escaping, table behaviour,
  etc.). Enumerated values use backed enums such as `HeadingStyle::ATX` or
  `CodeBlockStyle::TILDES`.
- `InlineImageConfig` controls inline image extraction (max decoded size,
  filename prefix, SVG capture, dimension inference).
- `PreprocessingOptions` configures sanitisation behaviour prior to conversion.

Each object provides a `toArray()` method and is serialisation-friendly.

## Inline image extraction

`Converter::convertWithInlineImages()` returns an `InlineImageExtraction`
instance containing:

- `markdown`: rendered Markdown string
- `inlineImages`: list of `InlineImage` objects exposing data, format,
  metadata, dimensions, source (`InlineImageSource` enum), and attributes
- `warnings`: list of `InlineImageWarning` objects describing recoverable
  issues (invalid payloads, skipped assets, etc.)

Use `InlineImageFormat` to inspect the image type and `InlineImageDimensions`
for width/height when available.

## Testing and quality

```bash
composer run lint   # phpstan (level max) + php-cs-fixer dry-run
composer run format # php-cs-fixer auto-fix
composer run test   # builds the extension and runs PHPUnit
```

The test runner compiles the Rust extension into `target/release` and loads it
when executing PHPUnit. Tests cover conversion parity, option parsing, warning
handling, and inline image extraction.

## Troubleshooting

- **Extension not found**: build with `cargo build -p html-to-markdown-php --release`
  and ensure the resulting library resides in `target/release`. Update
  `php.ini` to include `extension=html_to_markdown`.
- **Missing Rust toolchain**: install Rust via `rustup` and ensure `cargo` is on
  the `PATH`.
- **PIE install copies to `/html_to_markdown.so`**: set the extension dir
  explicitly, e.g. `PHP_EXTENSION_DIR=$(php-config --extension-dir) pie install goldziher/html-to-markdown`
  (PIE then writes to that directory, and you can enable via `extension=html_to_markdown`).
- **Composer permissions**: run Composer with `--no-interaction --no-progress`
  in CI to avoid prompts.

## Contributing

1. Install dependencies (`composer install`) and build the extension
   (`cargo build -p html-to-markdown-php --release`).
2. Run `composer run lint` and `composer run test`.
3. Follow the existing coding guidelines enforced by php-cs-fixer and phpstan.

## License

MIT © Na'aman Hirschfeld
