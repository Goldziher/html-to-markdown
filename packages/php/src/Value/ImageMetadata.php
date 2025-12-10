<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

final readonly class ImageMetadata
{
    /**
     * @param array<string, string> $attributes
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

        $dimensions = null;
        if (\array_key_exists('dimensions', $payload) && \is_array($payload['dimensions']) && \count($payload['dimensions']) === 2) {
            $dimensions = [(int) $payload['dimensions'][0], (int) $payload['dimensions'][1]];
        }

        return new self(
            src: (string) $payload['src'],
            alt: $payload['alt'] ?? null,
            title: $payload['title'] ?? null,
            dimensions: $dimensions,
            imageType: (string) $payload['image_type'],
            attributes: self::normalizeStringMap($payload['attributes'] ?? []),
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
     * @param mixed $map
     * @return array<string, string>
     */
    private static function normalizeStringMap($map): array
    {
        if (!is_array($map)) {
            return [];
        }

        $result = [];
        foreach ($map as $key => $value) {
            if (\is_string($key) && \is_string($value)) {
                $result[$key] = $value;
            }
        }

        return $result;
    }
}
