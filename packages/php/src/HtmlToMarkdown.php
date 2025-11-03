<?php

declare(strict_types=1);

namespace HtmlToMarkdown;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Service\Converter as ConverterService;
use HtmlToMarkdown\Value\InlineImageExtraction;

final class HtmlToMarkdown
{
    public static function convert(string $html, ?ConversionOptions $options = null): string
    {
        return ConverterService::create()->convert($html, $options);
    }

    public static function convertWithInlineImages(
        string $html,
        ?ConversionOptions $options = null,
        ?InlineImageConfig $config = null,
    ): InlineImageExtraction {
        return ConverterService::create()->convertWithInlineImages($html, $options, $config);
    }
}
