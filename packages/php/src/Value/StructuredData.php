<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

final readonly class StructuredData
{
    public function __construct(
        public string $dataType,
        public string $rawJson,
        public ?string $schemaType,
    ) {
    }

    /**
     * @param array<string, mixed> $payload
     */
    public static function fromExtensionPayload(array $payload): self
    {
        self::assertPayload($payload);

        return new self(
            dataType: (string) $payload['data_type'],
            rawJson: (string) $payload['raw_json'],
            schemaType: $payload['schema_type'] ?? null,
        );
    }

    /**
     * @param array<string, mixed> $payload
     */
    private static function assertPayload(array $payload): void
    {
        foreach (['data_type', 'raw_json'] as $required) {
            if (!\array_key_exists($required, $payload)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because("structured_data.$required", 'missing field in extension payload');
            }
        }
    }
}
