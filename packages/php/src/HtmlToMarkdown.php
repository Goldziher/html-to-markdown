<?php

declare(strict_types=1);

namespace HtmlToMarkdown;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Service\Converter as ConverterService;
use HtmlToMarkdown\Value\ExtendedMetadata;
use HtmlToMarkdown\Value\InlineImageExtraction;
use HtmlToMarkdown\Visitor\HtmlVisitor;

/**
 * @phpstan-import-type ConversionOptionsInput from \HtmlToMarkdown\Config\ConversionOptions
 * @phpstan-import-type InlineImageConfigInput from \HtmlToMarkdown\Config\InlineImageConfig
 */

final class HtmlToMarkdown
{
    /**
     * @param ConversionOptions|ConversionOptionsInput|null $options
     * @phpstan-param ConversionOptions|array<string, mixed>|null $options
     */
    public static function convert(string $html, ConversionOptions|array|null $options = null): string
    {
        return ConverterService::create()->convert($html, $options);
    }

    /**
     * @param ConversionOptions|ConversionOptionsInput|null $options
     * @param InlineImageConfig|InlineImageConfigInput|null $config
     * @phpstan-param ConversionOptions|array<string, mixed>|null $options
     * @phpstan-param InlineImageConfig|array<string, mixed>|null $config
     */
    public static function convertWithInlineImages(
        string $html,
        ConversionOptions|array|null $options = null,
        InlineImageConfig|array|null $config = null,
    ): InlineImageExtraction {
        return ConverterService::create()->convertWithInlineImages($html, $options, $config);
    }

    /**
     * @param ConversionOptions|ConversionOptionsInput|null $options
     * @param array<string, mixed>|null $metadataConfig
     * @phpstan-param ConversionOptions|array<string, mixed>|null $options
     * @phpstan-return array{markdown: string, metadata: ExtendedMetadata}
     */
    public static function convertWithMetadata(
        string $html,
        ConversionOptions|array|null $options = null,
        ?array $metadataConfig = null,
    ): array {
        return ConverterService::create()->convertWithMetadata($html, $options, $metadataConfig);
    }

    /**
     * Convert HTML with a custom visitor for advanced control.
     *
     * @param ConversionOptions|ConversionOptionsInput|null $options
     * @phpstan-param ConversionOptions|array<string, mixed>|null $options
     */
    public static function convertWithVisitor(
        string $html,
        ConversionOptions|array|null $options = null,
        ?HtmlVisitor $visitor = null,
    ): string {
        return ConverterService::create()->convertWithVisitor($html, $options, $visitor);
    }
}
