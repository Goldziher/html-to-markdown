# html_to_markdown PIE package

This directory contains the metadata and build helpers used to ship the
`html_to_markdown` PHP extension through [PIE](https://github.com/php/pie).

The actual extension source lives in the Rust workspace at the repository root;
`Cargo.toml` defines the crate `html-to-markdown-php`, which exports the PHP
module entry point.

## Local build

You can exercise the PIE build locally once `pie.phar` is on your `PATH`:

```bash
pie repository:add path $(pwd)
pie build html-to-markdown/html-to-markdown-ext:*@dev
```

The build requires a Rust toolchain (`cargo`) and the PHP development headers
for the target PHP version (typically provided by `phpize` from `php-dev`).

## Release packaging

`scripts/package_php_pie_source.sh` gathers this metadata together with the
Rust sources into a versioned tarball named
`php_html_to_markdown-<version>-src.tgz`. The release automation uploads the
tarball alongside pre-built Windows DLL archives so PIE can install the
extension on every platform.
