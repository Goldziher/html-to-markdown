<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Contract;

use HtmlToMarkdown\Visitor\HtmlVisitor;

/**
 * @phpstan-import-type ConversionOptionsInput from \HtmlToMarkdown\Config\ConversionOptions
 * @phpstan-import-type InlineImageConfigInput from \HtmlToMarkdown\Config\InlineImageConfig
 */

interface ExtensionBridge
{
    /**
     * @param ConversionOptionsInput|null $options
     */
    public function convert(string $html, ?array $options = null): string;

    /**
     * @param ConversionOptionsInput|null $options
     * @param InlineImageConfigInput|null $config
     *
     * @return array<string, mixed>
     */
    public function convertWithInlineImages(
        string $html,
        ?array $options = null,
        ?array $config = null,
    ): array;

    /**
     * @param ConversionOptionsInput|null $options
     * @param array<string, mixed>|null $metadataConfig
     *
     * @return array<string, mixed>
     */
    public function convertWithMetadata(
        string $html,
        ?array $options = null,
        ?array $metadataConfig = null,
    ): array;

    /**
     * @param ConversionOptionsInput|null $options
     */
    public function convertWithVisitor(
        string $html,
        ?array $options = null,
        ?HtmlVisitor $visitor = null,
    ): string;
}
