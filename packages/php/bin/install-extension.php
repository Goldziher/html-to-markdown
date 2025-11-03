<?php

declare(strict_types=1);

$extension = 'html_to_markdown';

if (extension_loaded($extension)) {
    return;
}

fwrite(
    STDERR,
    <<<EOT
The "{$extension}" extension is not currently enabled.

Install via PIE:
  pie install goldziher/html-to-markdown

Or download a prebuilt binary from the html-to-markdown release page and add it
to your PHP extension directory:
  https://github.com/Goldziher/html-to-markdown/releases

Once installed, enable the extension in your php.ini:
  extension={$extension}

EOT
);
