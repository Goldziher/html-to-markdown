<?php

declare(strict_types=1);

namespace HtmlToMarkdown;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Value\InlineImageExtraction;

function convert(string $html, ?ConversionOptions $options = null): string
{
    return HtmlToMarkdown::convert($html, $options);
}

function convert_with_inline_images(
    string $html,
    ?ConversionOptions $options = null,
    ?InlineImageConfig $config = null,
): InlineImageExtraction {
    return HtmlToMarkdown::convertWithInlineImages($html, $options, $config);
}
