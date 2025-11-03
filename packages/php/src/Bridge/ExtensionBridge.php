<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Bridge;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Contract\ExtensionBridge as ExtensionBridgeContract;
use HtmlToMarkdown\Exception\ConversionFailed;
use HtmlToMarkdown\Exception\ExtensionNotLoaded;
use HtmlToMarkdown\Exception\InvalidOption;
use HtmlToMarkdown\Value\InlineImageExtraction;

final class ExtensionBridge implements ExtensionBridgeContract
{
    private const CONVERT_FUNCTION = 'html_to_markdown_convert';
    private const CONVERT_INLINE_FUNCTION = 'html_to_markdown_convert_with_inline_images';

    public function convert(string $html, ?ConversionOptions $options = null): string
    {
        /** @var callable-string $callable */
        $callable = self::CONVERT_FUNCTION;
        if (!function_exists($callable)) {
            throw ExtensionNotLoaded::create();
        }

        $optionsArray = $options?->toArray();

        try {
            /** @var string $result */
            $result = $callable($html, $optionsArray);
        } catch (\Throwable $exception) {
            throw ConversionFailed::withMessage($exception->getMessage());
        }

        return $result;
    }

    public function convertWithInlineImages(
        string $html,
        ?ConversionOptions $options = null,
        ?InlineImageConfig $config = null,
    ): InlineImageExtraction {
        /** @var callable-string $callable */
        $callable = self::CONVERT_INLINE_FUNCTION;
        if (!function_exists($callable)) {
            throw ExtensionNotLoaded::create();
        }

        $optionsArray = $options?->toArray();
        $configArray = $config?->toArray();

        try {
            /** @var array<string, mixed> $payload */
            $payload = $callable($html, $optionsArray, $configArray);
        } catch (\Throwable $exception) {
            throw ConversionFailed::withMessage($exception->getMessage());
        }

        if (!is_array($payload)) {
            throw InvalidOption::because(
                'convert_with_inline_images',
                'extension returned unexpected payload',
            );
        }

        return InlineImageExtraction::fromExtensionPayload($payload);
    }
}
