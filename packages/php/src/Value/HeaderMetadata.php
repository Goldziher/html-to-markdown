<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

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
            level: (int) $payload['level'],
            text: (string) $payload['text'],
            id: $payload['id'] ?? null,
            depth: (int) $payload['depth'],
            htmlOffset: (int) $payload['html_offset'],
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
