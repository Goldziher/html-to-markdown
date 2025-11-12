<?php

declare(strict_types=1);

require __DIR__ . '/vendor/autoload.php';

use function HtmlToMarkdown\convert;

$html = '<h1>PHP Smoke Test</h1><p>Verifies the native extension.</p>';
$markdown = convert($html);

if (strpos($markdown, '# PHP Smoke Test') === false) {
    throw new RuntimeException('html-to-markdown did not return the expected heading');
}

echo "\033[32m✓ html-to-markdown (PHP) produced markdown\033[0m\n";
echo $markdown, "\n";
