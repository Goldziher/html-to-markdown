<?php

declare(strict_types=1);

namespace HtmlToMarkdown;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Service\Converter as ConverterService;
use HtmlToMarkdown\Value\InlineImageExtraction;

final class HtmlToMarkdown
{
    /**
     * @param ConversionOptions|array<string, mixed>|null $options
     */
    public static function convert(string $html, ConversionOptions|array|null $options = null): string
    {
        return ConverterService::create()->convert($html, $options);
    }

    /**
     * @param ConversionOptions|array<string, mixed>|null $options
     * @param InlineImageConfig|array<string, mixed>|null $config
     */
    public static function convertWithInlineImages(
        string $html,
        ConversionOptions|array|null $options = null,
        InlineImageConfig|array|null $config = null,
    ): InlineImageExtraction {
        return ConverterService::create()->convertWithInlineImages($html, $options, $config);
    }
}
