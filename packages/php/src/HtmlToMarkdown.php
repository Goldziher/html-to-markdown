<?php

declare(strict_types=1);

namespace HtmlToMarkdown;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Service\Converter as ConverterService;
use HtmlToMarkdown\Value\InlineImageExtraction;

/**
 * @phpstan-import-type ConversionOptionsInput from HtmlToMarkdown\Config\ConversionOptions
 * @phpstan-import-type InlineImageConfigInput from HtmlToMarkdown\Config\InlineImageConfig
 */

final class HtmlToMarkdown
{
    /**
     * @param ConversionOptions|ConversionOptionsInput|null $options
     */
    public static function convert(string $html, ConversionOptions|array|null $options = null): string
    {
        return ConverterService::create()->convert($html, $options);
    }

    /**
     * @param ConversionOptions|ConversionOptionsInput|null $options
     * @param InlineImageConfig|InlineImageConfigInput|null $config
     */
    public static function convertWithInlineImages(
        string $html,
        ConversionOptions|array|null $options = null,
        InlineImageConfig|array|null $config = null,
    ): InlineImageExtraction {
        return ConverterService::create()->convertWithInlineImages($html, $options, $config);
    }
}
