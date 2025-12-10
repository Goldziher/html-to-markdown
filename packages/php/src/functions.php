<?php

declare(strict_types=1);

namespace HtmlToMarkdown;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Value\ExtendedMetadata;
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

/**
 * @param ConversionOptions|ConversionOptionsInput|null $options
 * @param array<string, mixed>|null $metadataConfig
 * @return array{markdown: string, metadata: ExtendedMetadata}
 */
function convert_with_metadata(
    string $html,
    ConversionOptions|array|null $options = null,
    ?array $metadataConfig = null,
): array {
    return HtmlToMarkdown::convertWithMetadata($html, $options, $metadataConfig);
}
