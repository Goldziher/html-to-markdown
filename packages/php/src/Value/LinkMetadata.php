<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

use HtmlToMarkdown\Internal\TypeAssertions;

final readonly class LinkMetadata
{
    /**
     * @param list<string> $rel
     * @param array<string, string> $attributes
     */
    public function __construct(
        public string $href,
        public string $text,
        public ?string $title,
        public string $linkType,
        public array $rel,
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
            href: TypeAssertions::string($payload['href'], 'link_metadata.href'),
            text: TypeAssertions::string($payload['text'], 'link_metadata.text'),
            title: TypeAssertions::stringOrNull($payload['title'] ?? null, 'link_metadata.title'),
            linkType: TypeAssertions::string($payload['link_type'], 'link_metadata.link_type'),
            rel: TypeAssertions::stringList($payload['rel'], 'link_metadata.rel'),
            attributes: TypeAssertions::stringMap(
                $payload['attributes'] ?? [],
                'link_metadata.attributes',
            ),
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
}
