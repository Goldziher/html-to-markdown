# html_to_markdown PIE package

This directory contains the metadata and build helpers used to ship the
`html_to_markdown` PHP extension through [PIE](https://github.com/php/pie).

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

The actual extension source lives in the Rust workspace at the repository root;
`Cargo.toml` defines the crate `html-to-markdown-php`, which exports the PHP
module entry point.

## Usage

Once the PIE artifact is installed or the extension is compiled locally, the
global helper functions become available to PHP directly (and through the
Composer wrapper):

```php
<?php
use function HtmlToMarkdown\convert;
use function HtmlToMarkdown\convert_with_inline_images;

$markdown = convert('<h1>Hello</h1>');

[$markdown, $images, $warnings] = convert_with_inline_images(
    '<p><img src="data:image/png;base64,..." alt="Logo" /></p>'
);
```

For a typed API built on top of these primitives, install the
`html-to-markdown/extension` Composer package which layers enums, value objects,
and exceptions above the shared Rust engine.

## Local build

You can exercise the PIE build locally once `pie.phar` is on your `PATH`:

```bash
pie repository:add path $(pwd)
pie build goldziher/html-to-markdown:*@dev
```

The build requires a Rust toolchain (`cargo`) and the PHP development headers
for the target PHP version (typically provided by `phpize` from `php-dev`).

## Release packaging

`scripts/package_php_pie_source.sh` gathers this metadata together with the
Rust sources into a versioned tarball named
`php_html_to_markdown-<version>-src.tgz`. The release automation uploads the
tarball alongside pre-built Windows DLL archives so PIE can install the
extension on every platform.
