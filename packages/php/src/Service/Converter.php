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

    /**
     * @param ConversionOptions|array<string, mixed>|null $options
     */
    public function convert(string $html, ConversionOptions|array|null $options = null): string
    {
        return $this->bridge->convert($html, $this->normalizeOptions($options));
    }

    /**
     * @param ConversionOptions|array<string, mixed>|null $options
     * @param InlineImageConfig|array<string, mixed>|null $config
     */
    public function convertWithInlineImages(
        string $html,
        ConversionOptions|array|null $options = null,
        InlineImageConfig|array|null $config = null,
    ): InlineImageExtraction {
        $payload = $this->bridge->convertWithInlineImages(
            $html,
            $this->normalizeOptions($options),
            $this->normalizeImageConfig($config),
        );

        return InlineImageExtraction::fromExtensionPayload($payload);
    }

    /**
     * @param ConversionOptions|array<string, mixed>|null $options
     */
    /**
     * @param ConversionOptions|array<string, mixed>|null $options
     * @return array<string, mixed>|null
     */
    private function normalizeOptions(ConversionOptions|array|null $options): ?array
    {
        if ($options === null) {
            return null;
        }

        if ($options instanceof ConversionOptions) {
            return $options->toArray();
        }

        return ConversionOptions::fromArray($options)->toArray();
    }

    /**
     * @param InlineImageConfig|array<string, mixed>|null $config
     */
    /**
     * @param InlineImageConfig|array<string, mixed>|null $config
     * @return array<string, mixed>|null
     */
    private function normalizeImageConfig(InlineImageConfig|array|null $config): ?array
    {
        if ($config === null) {
            return null;
        }

        if ($config instanceof InlineImageConfig) {
            return $config->toArray();
        }

        return InlineImageConfig::fromArray($config)->toArray();
    }
}
