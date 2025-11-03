<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

use HtmlToMarkdown\Enum\InlineImageSource;
use HtmlToMarkdown\Exception\InvalidOption;

final readonly class InlineImage
{
    /**
     * @param array<string, string> $attributes
     */
    private function __construct(
        public string $data,
        public InlineImageFormat $format,
        public ?string $filename,
        public ?string $description,
        public ?InlineImageDimensions $dimensions,
        public InlineImageSource $source,
        public array $attributes,
    ) {
    }

    /**
     * @param array<string, mixed> $payload
     */
    public static function fromExtensionPayload(array $payload): self
    {
        self::assertPayload($payload);

        /** @var string $data */
        $data = $payload['data'];
        /** @var string $format */
        $format = $payload['format'];
        /** @var string $source */
        $source = $payload['source'];

        $normalized = self::normalizePayload($payload);

        return new self(
            data: $normalized['data'],
            format: InlineImageFormat::fromString($normalized['format']),
            filename: $normalized['filename'],
            description: $normalized['description'],
            dimensions: $normalized['dimensions'] !== null
                ? InlineImageDimensions::fromArray($normalized['dimensions'])
                : null,
            source: InlineImageSource::fromString($normalized['source']),
            attributes: $normalized['attributes'],
        );
    }

    /**
     * @param array<string, mixed> $payload
     */
    private static function assertPayload(array $payload): void
    {
        foreach (['data', 'format', 'source', 'attributes'] as $required) {
            if (!array_key_exists($required, $payload)) {
                throw InvalidOption::because("inline_image.$required", 'missing field in extension payload');
            }
        }

        if (!is_string($payload['data'])) {
            throw InvalidOption::because('inline_image.data', 'expected binary string');
        }

        if (!is_string($payload['format'])) {
            throw InvalidOption::because('inline_image.format', 'expected string');
        }

        if (array_key_exists('filename', $payload) && !in_array($payload['filename'], [null], true) && !is_string($payload['filename'])) {
            throw InvalidOption::because('inline_image.filename', 'expected string or null');
        }

        if (array_key_exists('description', $payload) && !in_array($payload['description'], [null], true) && !is_string($payload['description'])) {
            throw InvalidOption::because('inline_image.description', 'expected string or null');
        }

        if (array_key_exists('dimensions', $payload) && $payload['dimensions'] !== null) {
            if (!is_array($payload['dimensions']) || count($payload['dimensions']) !== 2) {
                throw InvalidOption::because('inline_image.dimensions', 'expected [width, height]');
            }
        }

        if (!is_string($payload['source'])) {
            throw InvalidOption::because('inline_image.source', 'expected string');
        }

        if (!is_array($payload['attributes'] ?? [])) {
            throw InvalidOption::because('inline_image.attributes', 'expected associative array');
        }

        foreach (($payload['attributes'] ?? []) as $key => $value) {
            if (!is_string($key) || !is_string($value)) {
                throw InvalidOption::because('inline_image.attributes', 'expected array<string,string>');
            }
        }
    }

    /**
     * @param array<string, mixed> $payload
     * @return array{
     *   data: string,
     *   format: string,
     *   filename: ?string,
     *   description: ?string,
     *   dimensions: ?array{0:int,1:int},
     *   source: string,
     *   attributes: array<string,string>
     * }
     */
    private static function normalizePayload(array $payload): array
    {
        self::assertPayload($payload);

        /** @var string $data */
        $data = $payload['data'];
        /** @var string $format */
        $format = $payload['format'];
        /** @var string $source */
        $source = $payload['source'];

        $filename = array_key_exists('filename', $payload) ? ($payload['filename'] ?? null) : null;
        if ($filename !== null && !is_string($filename)) {
            throw InvalidOption::because('inline_image.filename', 'expected string or null');
        }

        $description = array_key_exists('description', $payload) ? ($payload['description'] ?? null) : null;
        if ($description !== null && !is_string($description)) {
            throw InvalidOption::because('inline_image.description', 'expected string or null');
        }

        $dimensionsRaw = array_key_exists('dimensions', $payload) ? ($payload['dimensions'] ?? null) : null;
        if ($dimensionsRaw !== null && !is_array($dimensionsRaw)) {
            throw InvalidOption::because('inline_image.dimensions', 'expected [width, height]');
        }

        $dimensions = null;
        if ($dimensionsRaw !== null) {
            if (!array_key_exists(0, $dimensionsRaw) || !array_key_exists(1, $dimensionsRaw)) {
                throw InvalidOption::because('inline_image.dimensions', 'expected [width, height]');
            }

            $dimensions = [
                (int) $dimensionsRaw[0],
                (int) $dimensionsRaw[1],
            ];
        }

        $attributesRaw = $payload['attributes'] ?? [];
        if (!is_array($attributesRaw)) {
            throw InvalidOption::because('inline_image.attributes', 'expected associative array');
        }

        $attributes = [];
        foreach ($attributesRaw as $key => $value) {
            if (!is_string($key) || !is_string($value)) {
                throw InvalidOption::because('inline_image.attributes', 'expected array<string,string>');
            }

            $attributes[$key] = $value;
        }

        return [
            'data' => $data,
            'format' => $format,
            'filename' => $filename,
            'description' => $description,
            'dimensions' => $dimensions,
            'source' => $source,
            'attributes' => $attributes,
        ];
    }
}
