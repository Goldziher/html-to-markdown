<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Service;

use HtmlToMarkdown\Bridge\ExtensionBridge;
use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Contract\ExtensionBridge as ExtensionBridgeContract;
use HtmlToMarkdown\Internal\TypeAssertions;
use HtmlToMarkdown\Value\ExtendedMetadata;
use HtmlToMarkdown\Value\InlineImageExtraction;

/**
 * @phpstan-import-type ConversionOptionsInput from \HtmlToMarkdown\Config\ConversionOptions
 * @phpstan-import-type InlineImageConfigInput from \HtmlToMarkdown\Config\InlineImageConfig
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
     * @phpstan-param ConversionOptions|array<string, mixed>|null $options
     */
    public function convert(string $html, ConversionOptions|array|null $options = null): string
    {
        return $this->bridge->convert($html, $this->normalizeOptions($options));
    }

    /**
     * @param ConversionOptions|ConversionOptionsInput|null $options
     * @param InlineImageConfig|InlineImageConfigInput|null $config
     * @phpstan-param ConversionOptions|array<string, mixed>|null $options
     * @phpstan-param InlineImageConfig|array<string, mixed>|null $config
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
     * @param array<string, mixed>|null $metadataConfig
     * @phpstan-param ConversionOptions|array<string, mixed>|null $options
     * @phpstan-return array{markdown: string, metadata: ExtendedMetadata}
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

        $markdown = TypeAssertions::string($payload['markdown'] ?? '', 'convert_with_metadata.markdown');
        $metadataPayload = \is_array($payload['metadata'] ?? null) ? $payload['metadata'] : [];

        return [
            'markdown' => $markdown,
            'metadata' => ExtendedMetadata::fromExtensionPayload($metadataPayload),
        ];
    }

    /**
     * @phpstan-param ConversionOptions|array<string, mixed>|null $options
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
     * @phpstan-param InlineImageConfig|array<string, mixed>|null $config
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
