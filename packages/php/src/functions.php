<?php

declare(strict_types=1);

namespace HtmlToMarkdown;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Value\InlineImageExtraction;

/**
 * @phpstan-import-type ConversionOptionsInput from HtmlToMarkdown\Config\ConversionOptions
 * @phpstan-import-type InlineImageConfigInput from HtmlToMarkdown\Config\InlineImageConfig
 */

/**
 * @param ConversionOptions|ConversionOptionsInput|null $options
 */
function convert(string $html, ConversionOptions|array|null $options = null): string
{
    return HtmlToMarkdown::convert($html, $options);
}

/**
 * @param ConversionOptions|ConversionOptionsInput|null $options
 * @param InlineImageConfig|InlineImageConfigInput|null $config
 */
function convert_with_inline_images(
    string $html,
    ConversionOptions|array|null $options = null,
    InlineImageConfig|array|null $config = null,
): InlineImageExtraction {
    return HtmlToMarkdown::convertWithInlineImages($html, $options, $config);
}
