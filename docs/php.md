# PHP Extension

[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)

The PHP binding exposes the native Rust converter through the
`html_to_markdown` extension and a typed Composer package hosted in
`packages/php`. Install via PIE to get prebuilt binaries, or build locally with
`cargo`.

## Installation

Install the extension and high-level Composer wrapper:

Install the PHP package and ensure the native extension is available:

```bash
pie install goldziher/html-to-markdown
composer require html-to-markdown/extension
```

> **Note**
> Distribute the extension via PIE and publish prebuilt binaries as needed. Composer only wraps
the extension and provides the modern PHP surface area.

### Using PIE without a local build

```bash
pie install goldziher/html-to-markdown --install-project
```

The `--install-project` flag tells PIE to copy the extension built during the project installation (our bundled artifact) straight into your PHP install, so you don't need a system Rust toolchain.

## Distribution via PIE

The PHP extension is now packaged for [PIE](https://github.com/php/pie).
Each release tag uploads a `php_html_to_markdown-<version>-src.tgz` archive to
the GitHub release, which pie can consume directly:

```bash
pie install goldziher/html-to-markdown
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
`InlineImageWarning` instances that map 1:1 with the Rust API.

### Building locally on Windows

If you want to compile the extension on Windows rather than relying on the
published DLLs, make sure the following prerequisites are in place:

- **Official PHP SDK** — cargo-php only supports PHP installations sourced from
  <https://windows.php.net>. Custom SDKs are not yet supported.
- **Rust nightly toolchain** — vectorcall support is an unstable nightly feature.
  You can install it with `rustup toolchain install nightly` and set
  `RUSTUP_TOOLCHAIN=nightly` before invoking `cargo-php`.
- **LLD linker** — the recommended configuration uses the bundled `rust-lld`.
  Create a `.cargo/config.toml` alongside the extension sources with:

  ```toml
  [target.x86_64-pc-windows-msvc]
  linker = "rust-lld"

  [target.i686-pc-windows-msvc]
  linker = "rust-lld"
  ```

  Microsoft’s `link.exe` is technically supported but tends to be brittle
  across SDK revisions.
- **MSVC build tools** — the `cc` crate expects `cl.exe` to be available
  (installed with Visual Studio or the Build Tools workload).

When targeting debug builds, you must point `PHP_LIB` at a debug PHP SDK (for
example `set PHP_LIB=C:\php-sdk\php-dev\vc16\x64\php-8.3.13-src\x64\Debug_TS`).

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

## See also

- [Language binding overview](./bindings.md) for the full matrix of supported runtimes.
- [Rust crate documentation](../crates/html-to-markdown/README.md) for the core engine.
