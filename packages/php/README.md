# html-to-markdown PHP package

Typed PHP bindings for the `html_to_markdown` native extension generated from
the Rust html-to-markdown engine. The API mirrors the behaviour exposed by the
Rust, Python, Ruby, Node.js, and WASM distributions while embracing modern PHP
features (readonly value objects, enums, typed exceptions).

## Requirements

- PHP 8.2 or newer
- `html_to_markdown` native extension (install via PECL or download the release
  shared library)
- Composer for dependency management

## Installation

```bash
composer require html-to-markdown/extension
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
