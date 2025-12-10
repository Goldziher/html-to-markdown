<?php

declare(strict_types=1);

namespace HtmlToMarkdown;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Value\ExtendedMetadata;
use HtmlToMarkdown\Value\InlineImageExtraction;

/**
 * @phpstan-import-type ConversionOptionsInput from \HtmlToMarkdown\Config\ConversionOptions
 * @phpstan-import-type InlineImageConfigInput from \HtmlToMarkdown\Config\InlineImageConfig
 */

/**
 * @param ConversionOptions|array<string, mixed>|null $options
 * @phpstan-param ConversionOptions|array<string, mixed>|null $options
 */
function convert(string $html, ConversionOptions|array|null $options = null): string
{
    return HtmlToMarkdown::convert($html, $options);
}

/**
 * @param ConversionOptions|array<string, mixed>|null $options
 * @param InlineImageConfig|array<string, mixed>|null $config
 * @phpstan-param ConversionOptions|array<string, mixed>|null $options
 * @phpstan-param InlineImageConfig|array<string, mixed>|null $config
 */
function convert_with_inline_images(
    string $html,
    ConversionOptions|array|null $options = null,
    InlineImageConfig|array|null $config = null,
): InlineImageExtraction {
    return HtmlToMarkdown::convertWithInlineImages($html, $options, $config);
}

/**
 * @param ConversionOptions|array<string, mixed>|null $options
 * @param array<string, bool|string>|null $metadataConfig
 * @phpstan-param ConversionOptions|array<string, mixed>|null $options
 * @return array{markdown: string, metadata: ExtendedMetadata}
 */
function convert_with_metadata(
    string $html,
    ConversionOptions|array|null $options = null,
    ?array $metadataConfig = null,
): array {
    return HtmlToMarkdown::convertWithMetadata($html, $options, $metadataConfig);
}
