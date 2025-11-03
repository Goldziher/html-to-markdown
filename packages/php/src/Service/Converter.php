<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Service;

use HtmlToMarkdown\Bridge\ExtensionBridge;
use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Contract\ExtensionBridge as ExtensionBridgeContract;
use HtmlToMarkdown\Value\InlineImageExtraction;

final class Converter
{
    public function __construct(
        private readonly ExtensionBridgeContract $bridge,
    ) {
    }

    public static function create(): self
    {
        return new self(new ExtensionBridge());
    }

    public function convert(string $html, ?ConversionOptions $options = null): string
    {
        return $this->bridge->convert($html, $options);
    }

    public function convertWithInlineImages(
        string $html,
        ?ConversionOptions $options = null,
        ?InlineImageConfig $config = null,
    ): InlineImageExtraction {
        return $this->bridge->convertWithInlineImages($html, $options, $config);
    }
}
