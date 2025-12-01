# PHP Smoke Test

Exercises the Composer helpers once the `html_to_markdown` extension is
installed.

## 0. Ensure the extension is available

```bash
cargo build -p html-to-markdown-php --release
# or
pie install goldziher/html-to-markdown
```

Add `extension=html_to_markdown` to `php.ini` so Composer can load it.

## 1. Test the latest Packagist release

```bash
cd examples/php-smoke
composer install
php convert.php
```

## 2. Test the local Composer package

```bash
cd examples/php-smoke
composer config repositories.local path ../../packages/php
composer update goldziher/html-to-markdown --prefer-source
php convert.php
```

Remove the temporary repository entry when done:

```bash
composer config --unset repositories.local
```
