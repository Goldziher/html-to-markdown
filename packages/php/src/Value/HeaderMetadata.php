<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

use HtmlToMarkdown\Internal\TypeAssertions;

final readonly class HeaderMetadata
{
    public function __construct(
        public int $level,
        public string $text,
        public ?string $id,
        public int $depth,
        public int $htmlOffset,
    ) {
    }

    /**
     * @param array<string, mixed> $payload
     */
    public static function fromExtensionPayload(array $payload): self
    {
        self::assertPayload($payload);

        return new self(
            level: TypeAssertions::positiveInt($payload['level'], 'header_metadata.level'),
            text: TypeAssertions::string($payload['text'], 'header_metadata.text'),
            id: TypeAssertions::stringOrNull($payload['id'] ?? null, 'header_metadata.id'),
            depth: TypeAssertions::positiveInt($payload['depth'], 'header_metadata.depth'),
            htmlOffset: TypeAssertions::positiveInt($payload['html_offset'], 'header_metadata.html_offset'),
        );
    }

    /**
     * @param array<string, mixed> $payload
     */
    private static function assertPayload(array $payload): void
    {
        foreach (['level', 'text', 'depth', 'html_offset'] as $required) {
            if (!\array_key_exists($required, $payload)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because("header_metadata.$required", 'missing field in extension payload');
            }
        }
    }
}
