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

## Distribution via PIE

The PHP extension is now packaged for [PIE](https://github.com/php/pie).
Each release tag uploads a `php_html_to_markdown-<version>-src.tgz` archive to
the GitHub release, which pie can consume directly:

```bash
pie install html-to-markdown/html-to-markdown-ext
```

The install process requires a Rust toolchain (`cargo`) and the PHP development
headers for the target PHP version (available via `phpize`).

### Windows DLLs

The build pipeline also pushes pre-built Windows binaries following the PIE
naming convention:

```
php_html_to_markdown-<tag>-<php-version>-<ts|nts>-<compiler>-<arch>.zip
```

Each archive contains:

- `php_html_to_markdown.dll` — the extension library
- `php_html_to_markdown.pdb` — debug symbols (when available)
- any dependent DLLs required by the build and associated licenses

These ZIP files are attached to the same GitHub release and can be consumed by
PIE on Windows or installed manually by copying the DLL into the appropriate
`php/ext` directory and enabling `extension=php_html_to_markdown` in `php.ini`.

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
