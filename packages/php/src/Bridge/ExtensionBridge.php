<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Bridge;

use HtmlToMarkdown\Contract\ExtensionBridge as ExtensionBridgeContract;
use HtmlToMarkdown\Exception\ConversionFailed;
use HtmlToMarkdown\Exception\ExtensionNotLoaded;
use HtmlToMarkdown\Exception\InvalidOption;

/**
 * @phpstan-import-type ConversionOptionsInput from HtmlToMarkdown\Config\ConversionOptions
 * @phpstan-import-type InlineImageConfigInput from HtmlToMarkdown\Config\InlineImageConfig
 */

final class ExtensionBridge implements ExtensionBridgeContract
{
    private const CONVERT_FUNCTION = 'html_to_markdown_convert';
    private const CONVERT_INLINE_FUNCTION = 'html_to_markdown_convert_with_inline_images';

    /**
     * @param ConversionOptionsInput|null $options
     */
    public function convert(string $html, ?array $options = null): string
    {
        /** @var callable-string $callable */
        $callable = self::CONVERT_FUNCTION;
        if (!\function_exists($callable)) {
            throw ExtensionNotLoaded::create();
        }

        try {
            /** @var string $result */
            $result = $callable($html, $options);
        } catch (\Throwable $exception) {
            throw ConversionFailed::withMessage($exception->getMessage());
        }

        return $result;
    }

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
    ): array {
        /** @var callable-string $callable */
        $callable = self::CONVERT_INLINE_FUNCTION;
        if (!\function_exists($callable)) {
            throw ExtensionNotLoaded::create();
        }

        try {
            /** @var array<string, mixed> $payload */
            $payload = $callable($html, $options, $config);
        } catch (\Throwable $exception) {
            throw ConversionFailed::withMessage($exception->getMessage());
        }

        if (!\is_array($payload)) {
            throw InvalidOption::because(
                'convert_with_inline_images',
                'extension returned unexpected payload',
            );
        }

        return $payload;
    }
}
