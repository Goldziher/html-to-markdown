<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Service;

use HtmlToMarkdown\Bridge\ExtensionBridge;
use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Contract\ExtensionBridge as ExtensionBridgeContract;
use HtmlToMarkdown\Value\ExtendedMetadata;
use HtmlToMarkdown\Value\InlineImageExtraction;

/**
 * @phpstan-import-type ConversionOptionsInput from HtmlToMarkdown\Config\ConversionOptions
 * @phpstan-import-type InlineImageConfigInput from HtmlToMarkdown\Config\InlineImageConfig
 */

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
     * @param ConversionOptions|ConversionOptionsInput|null $options
     */
    public function convert(string $html, ConversionOptions|array|null $options = null): string
    {
        return $this->bridge->convert($html, $this->normalizeOptions($options));
    }

    /**
     * @param ConversionOptions|ConversionOptionsInput|null $options
     * @param InlineImageConfig|InlineImageConfigInput|null $config
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
     * @param ConversionOptions|ConversionOptionsInput|null $options
     * @param array<string, mixed>|null $metadataConfig
     */
    public function convertWithMetadata(
        string $html,
        ConversionOptions|array|null $options = null,
        ?array $metadataConfig = null,
    ): array {
        $payload = $this->bridge->convertWithMetadata(
            $html,
            $this->normalizeOptions($options),
            $metadataConfig,
        );

        return [
            'markdown' => $payload['markdown'] ?? '',
            'metadata' => ExtendedMetadata::fromExtensionPayload($payload['metadata'] ?? []),
        ];
    }

    /**
     * @param ConversionOptions|ConversionOptionsInput|null $options
     * @phpstan-return ConversionOptionsInput|null
     */
    private function normalizeOptions(ConversionOptions|array|null $options): ?array
    {
        if ($options === null) {
            return null;
        }

        if ($options instanceof ConversionOptions) {
            $payload = $options->toArray();
        } else {
            $payload = ConversionOptions::fromArray($options)->toArray();
        }

        return $payload === [] ? null : $payload;
    }

    /**
     * @param InlineImageConfig|InlineImageConfigInput|null $config
     * @phpstan-return InlineImageConfigInput|null
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
