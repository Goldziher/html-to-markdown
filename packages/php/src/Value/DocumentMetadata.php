<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

final readonly class DocumentMetadata
{
    /**
     * @param array<string, string> $openGraph
     * @param array<string, string> $twitterCard
     * @param array<string, string> $metaTags
     */
    public function __construct(
        public ?string $title,
        public ?string $description,
        public array $keywords,
        public ?string $author,
        public ?string $canonicalUrl,
        public ?string $baseHref,
        public ?string $language,
        public ?string $textDirection,
        public array $openGraph,
        public array $twitterCard,
        public array $metaTags,
    ) {
    }

    /**
     * @param array<string, mixed> $payload
     */
    public static function fromExtensionPayload(array $payload): self
    {
        self::assertPayload($payload);

        return new self(
            title: $payload['title'] ?? null,
            description: $payload['description'] ?? null,
            keywords: \is_array($payload['keywords'] ?? null) ? $payload['keywords'] : [],
            author: $payload['author'] ?? null,
            canonicalUrl: $payload['canonical_url'] ?? null,
            baseHref: $payload['base_href'] ?? null,
            language: $payload['language'] ?? null,
            textDirection: $payload['text_direction'] ?? null,
            openGraph: self::normalizeStringMap($payload['open_graph'] ?? []),
            twitterCard: self::normalizeStringMap($payload['twitter_card'] ?? []),
            metaTags: self::normalizeStringMap($payload['meta_tags'] ?? []),
        );
    }

    /**
     * @param array<string, mixed> $payload
     */
    private static function assertPayload(array $payload): void
    {
        foreach (['title', 'description', 'author', 'canonical_url', 'base_href', 'language', 'text_direction', 'keywords', 'open_graph', 'twitter_card', 'meta_tags'] as $field) {
            if (!\array_key_exists($field, $payload)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because("document_metadata.$field", 'missing field in extension payload');
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
