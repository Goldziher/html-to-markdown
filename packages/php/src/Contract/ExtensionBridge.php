<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Contract;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Value\InlineImageExtraction;

interface ExtensionBridge
{
    public function convert(string $html, ?ConversionOptions $options = null): string;

    public function convertWithInlineImages(
        string $html,
        ?ConversionOptions $options = null,
        ?InlineImageConfig $config = null,
    ): InlineImageExtraction;
}
