# html-to-markdown PHP package

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg)](https://crates.io/crates/html-to-markdown-rs)
[![npm version](https://badge.fury.io/js/html-to-markdown-node.svg)](https://www.npmjs.com/package/html-to-markdown-node)
[![PyPI version](https://badge.fury.io/py/html-to-markdown.svg)](https://pypi.org/project/html-to-markdown/)
[![Gem Version](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)

Typed PHP bindings for the `html_to_markdown` native extension generated from
the Rust html-to-markdown engine. The API mirrors the behaviour exposed by the
Rust, Python, Ruby, Node.js, and WASM distributions while embracing modern PHP
features (readonly value objects, enums, typed exceptions).

## Requirements

- PHP 8.2 or newer
- `html_to_markdown` native extension (install via PIE `goldziher/html-to-markdown` or download the GitHub release artifacts)
- Composer for dependency management

## Installation

```bash
pie install goldziher/html-to-markdown        # Native extension via PIE
composer require html-to-markdown/extension   # Typed PHP API wrappers
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
- **Composer permissions**: run Composer with `--no-interaction --no-progress`
  in CI to avoid prompts.

## Contributing

1. Install dependencies (`composer install`) and build the extension
   (`cargo build -p html-to-markdown-php --release`).
2. Run `composer run lint` and `composer run test`.
3. Follow the existing coding guidelines enforced by php-cs-fixer and phpstan.

## License

MIT © Na'aman Hirschfeld
