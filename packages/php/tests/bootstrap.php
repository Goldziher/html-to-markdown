<?php

declare(strict_types=1);

if (!\extension_loaded('html_to_markdown')) {
    throw new RuntimeException('The html_to_markdown extension must be loaded before running tests.');
}

require_once __DIR__ . '/../vendor/autoload.php';
