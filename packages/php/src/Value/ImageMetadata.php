<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

use HtmlToMarkdown\Internal\TypeAssertions;

final readonly class ImageMetadata
{
    /**
     * @param array<string, string> $attributes
     * @param array{0:int,1:int}|null $dimensions
     */
    public function __construct(
        public string $src,
        public ?string $alt,
        public ?string $title,
        public ?array $dimensions,
        public string $imageType,
        public array $attributes,
    ) {
    }

    /**
     * @param array<string, mixed> $payload
     */
    public static function fromExtensionPayload(array $payload): self
    {
        self::assertPayload($payload);

        $dimensions = self::normalizeDimensions($payload['dimensions'] ?? null);

        return new self(
            src: TypeAssertions::string($payload['src'], 'image_metadata.src'),
            alt: TypeAssertions::stringOrNull($payload['alt'] ?? null, 'image_metadata.alt'),
            title: TypeAssertions::stringOrNull($payload['title'] ?? null, 'image_metadata.title'),
            dimensions: $dimensions,
            imageType: TypeAssertions::string($payload['image_type'], 'image_metadata.image_type'),
            attributes: TypeAssertions::stringMap($payload['attributes'] ?? [], 'image_metadata.attributes'),
        );
    }

    /**
     * @param array<string, mixed> $payload
     */
    private static function assertPayload(array $payload): void
    {
        foreach (['src', 'image_type', 'attributes'] as $required) {
            if (!\array_key_exists($required, $payload)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because("image_metadata.$required", 'missing field in extension payload');
            }
        }
    }

    /**
     * @param mixed $value
     * @return array{0:int,1:int}|null
     */
    private static function normalizeDimensions($value): ?array
    {
        if ($value === null) {
            return null;
        }

        if (!\is_array($value) || \count($value) !== 2) {
            throw \HtmlToMarkdown\Exception\InvalidOption::because(
                'image_metadata.dimensions',
                'expected list of two integers',
            );
        }

        return [
            TypeAssertions::positiveInt($value[0] ?? null, 'image_metadata.dimensions.0'),
            TypeAssertions::positiveInt($value[1] ?? null, 'image_metadata.dimensions.1'),
        ];
    }
}
