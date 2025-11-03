<?php

declare(strict_types=1);

namespace HtmlToMarkdown;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Value\InlineImageExtraction;

/**
 * @param ConversionOptions|array<string, mixed>|null $options
 */
function convert(string $html, ConversionOptions|array|null $options = null): string
{
    return HtmlToMarkdown::convert($html, $options);
}

/**
 * @param ConversionOptions|array<string, mixed>|null $options
 * @param InlineImageConfig|array<string, mixed>|null $config
 */
function convert_with_inline_images(
    string $html,
    ConversionOptions|array|null $options = null,
    InlineImageConfig|array|null $config = null,
): InlineImageExtraction {
    return HtmlToMarkdown::convertWithInlineImages($html, $options, $config);
}
