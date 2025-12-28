<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

use HtmlToMarkdown\Internal\TypeAssertions;

final readonly class DocumentMetadata
{
    /**
     * @param array<string, string> $openGraph
     * @param array<string, string> $twitterCard
     * @param array<string, string> $metaTags
     * @param list<string> $keywords
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
        return new self(
            title: TypeAssertions::stringOrNull($payload['title'] ?? null, 'document_metadata.title'),
            description: TypeAssertions::stringOrNull($payload['description'] ?? null, 'document_metadata.description'),
            keywords: \array_key_exists('keywords', $payload)
                ? TypeAssertions::stringList($payload['keywords'], 'document_metadata.keywords')
                : [],
            author: TypeAssertions::stringOrNull($payload['author'] ?? null, 'document_metadata.author'),
            canonicalUrl: TypeAssertions::stringOrNull(
                $payload['canonical_url'] ?? null,
                'document_metadata.canonical_url',
            ),
            baseHref: TypeAssertions::stringOrNull($payload['base_href'] ?? null, 'document_metadata.base_href'),
            language: TypeAssertions::stringOrNull($payload['language'] ?? null, 'document_metadata.language'),
            textDirection: TypeAssertions::stringOrNull(
                $payload['text_direction'] ?? null,
                'document_metadata.text_direction',
            ),
            openGraph: TypeAssertions::stringMap($payload['open_graph'] ?? [], 'document_metadata.open_graph'),
            twitterCard: TypeAssertions::stringMap($payload['twitter_card'] ?? [], 'document_metadata.twitter_card'),
            metaTags: TypeAssertions::stringMap($payload['meta_tags'] ?? [], 'document_metadata.meta_tags'),
        );
    }
}
