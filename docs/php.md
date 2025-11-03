# PHP Extension

The PHP bindings expose the native Rust converter through the `html_to_markdown`
extension and a typed composer package located in `packages/php`.

## Installation

Install the PHP package and ensure the native extension is available:

```bash
composer require html-to-markdown/extension
```

> **Note**
> Publish the extension via PECL or ship prebuilt binaries. Composer only wraps
the extension and provides the modern PHP surface area.

## Usage

```php
use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Service\Converter;
use function HtmlToMarkdown\convert;

$converter = Converter::create();

$markdown = $converter->convert('<h1>Hello</h1>');

$extraction = $converter->convertWithInlineImages(
    '<img src="data:image/png;base64,Zm9v" alt="demo">',
    new ConversionOptions(),
    new InlineImageConfig(inferDimensions: true),
);

// Procedural helpers delegate to the converter
$markdown = convert('<p>Lorem ipsum</p>');
```

Configuration is handled through `ConversionOptions`, `InlineImageConfig`, and
`PreprocessingOptions` value objects. Inline image extraction returns
`InlineImageExtraction`, containing `InlineImage` descriptors and
`InlineImageWarning` instances that match the Rust API one-to-one.

## Local testing

```bash
# Install composer dependencies
composer install --no-interaction --no-progress

# Build the extension and run the typed test suite
composer run lint
composer run test
```

`composer run test` invokes `cargo build -p html-to-markdown-php --release` to
ensure the latest extension is available before running PHPUnit.
