<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

final readonly class LinkMetadata
{
    /**
     * @param array<string, string> $attributes
     */
    public function __construct(
        public string $href,
        public string $text,
        public ?string $title,
        public string $linkType,
        public string $rel,
        public array $attributes,
    ) {
    }

    /**
     * @param array<string, mixed> $payload
     */
    public static function fromExtensionPayload(array $payload): self
    {
        self::assertPayload($payload);

        return new self(
            href: (string) $payload['href'],
            text: (string) $payload['text'],
            title: $payload['title'] ?? null,
            linkType: (string) $payload['link_type'],
            rel: (string) $payload['rel'],
            attributes: self::normalizeStringMap($payload['attributes'] ?? []),
        );
    }

    /**
     * @param array<string, mixed> $payload
     */
    private static function assertPayload(array $payload): void
    {
        foreach (['href', 'text', 'link_type', 'rel', 'attributes'] as $required) {
            if (!\array_key_exists($required, $payload)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because("link_metadata.$required", 'missing field in extension payload');
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
