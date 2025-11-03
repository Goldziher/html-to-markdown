# html-to-markdown PHP integration

Modern PHP bindings for the `html_to_markdown` native extension generated from
the Rust html-to-markdown engine. The package provides a typed, immutable API
that mirrors the behaviour exposed in the Rust, Python, Ruby, Node.js, and WASM
bindings.

## Installation

```bash
composer require html-to-markdown/extension
```

The package assumes the `html_to_markdown` extension is available (install via
PECL or download the release binary). Composer runs a small post-install script
that reminds you to enable the extension if it is missing.

## Usage

```php
use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\HtmlToMarkdown;
use HtmlToMarkdown\Service\Converter;
use function HtmlToMarkdown\convert;
use function HtmlToMarkdown\convert_with_inline_images;

// Object-oriented
$converter = Converter::create();
$markdown = $converter->convert('<h1>Hello</h1>');

$extraction = $converter->convertWithInlineImages(
    '<img src="data:image/png;base64,Zm9v" alt="demo">',
    new ConversionOptions(),
    new InlineImageConfig(inferDimensions: true),
);

// Procedural helpers
$markdown = convert('<p>Lorem ipsum</p>');
$extraction = convert_with_inline_images('<img src="data:image/png;base64,Zm9v" alt="demo">');
```

`ConversionOptions`, `InlineImageConfig`, and `PreprocessingOptions` are
immutable value objects with enum-backed fields mirroring the Rust configuration
struct. `convert_with_inline_images()` returns an `InlineImageExtraction`
instance containing the markdown output, inline image descriptors, and any
warnings emitted by the native converter.

## Testing

```bash
composer run lint   # phpstan + php-cs-fixer (dry-run)
composer run test   # builds the extension and executes PHPUnit
```
