<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

final readonly class ExtendedMetadata
{
    /**
     * @param list<HeaderMetadata> $headers
     * @param list<LinkMetadata> $links
     * @param list<ImageMetadata> $images
     * @param list<StructuredData> $structuredData
     */
    public function __construct(
        public DocumentMetadata $document,
        public array $headers,
        public array $links,
        public array $images,
        public array $structuredData,
    ) {
    }

    /**
     * @param array<string, mixed> $payload
     */
    public static function fromExtensionPayload(array $payload): self
    {
        $documentPayload = \is_array($payload['document'] ?? null) ? $payload['document'] : [];

        $headersPayload = \is_array($payload['headers'] ?? null) ? $payload['headers'] : [];

        $headers = [];
        foreach ($headersPayload as $header) {
            if (!\is_array($header)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because(
                    'extended_metadata.headers[]',
                    'expected array, got ' . \get_debug_type($header),
                );
            }

            $headers[] = HeaderMetadata::fromExtensionPayload($header);
        }

        $linksPayload = \is_array($payload['links'] ?? null) ? $payload['links'] : [];

        $links = [];
        foreach ($linksPayload as $link) {
            if (!\is_array($link)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because(
                    'extended_metadata.links[]',
                    'expected array, got ' . \get_debug_type($link),
                );
            }

            $links[] = LinkMetadata::fromExtensionPayload($link);
        }

        $imagesPayload = \is_array($payload['images'] ?? null) ? $payload['images'] : [];

        $images = [];
        foreach ($imagesPayload as $image) {
            if (!\is_array($image)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because(
                    'extended_metadata.images[]',
                    'expected array, got ' . \get_debug_type($image),
                );
            }

            $images[] = ImageMetadata::fromExtensionPayload($image);
        }

        $structuredDataPayload = \is_array($payload['structured_data'] ?? null) ? $payload['structured_data'] : [];

        $structuredData = [];
        foreach ($structuredDataPayload as $data) {
            if (!\is_array($data)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because(
                    'extended_metadata.structured_data[]',
                    'expected array, got ' . \get_debug_type($data),
                );
            }

            $structuredData[] = StructuredData::fromExtensionPayload($data);
        }

        return new self(
            document: DocumentMetadata::fromExtensionPayload($documentPayload),
            headers: $headers,
            links: $links,
            images: $images,
            structuredData: $structuredData,
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        return [
            'document' => [
                'title' => $this->document->title,
                'description' => $this->document->description,
                'keywords' => $this->document->keywords,
                'author' => $this->document->author,
                'canonical_url' => $this->document->canonicalUrl,
                'base_href' => $this->document->baseHref,
                'language' => $this->document->language,
                'text_direction' => $this->document->textDirection,
                'open_graph' => $this->document->openGraph,
                'twitter_card' => $this->document->twitterCard,
                'meta_tags' => $this->document->metaTags,
            ],
            'headers' => \array_map(
                static fn (HeaderMetadata $h): array => [
                    'level' => $h->level,
                    'text' => $h->text,
                    'id' => $h->id,
                    'depth' => $h->depth,
                    'html_offset' => $h->htmlOffset,
                ],
                $this->headers,
            ),
            'links' => \array_map(
                static fn (LinkMetadata $l): array => [
                    'href' => $l->href,
                    'text' => $l->text,
                    'title' => $l->title,
                    'link_type' => $l->linkType,
                    'rel' => $l->rel,
                    'attributes' => $l->attributes,
                ],
                $this->links,
            ),
            'images' => \array_map(
                static fn (ImageMetadata $i): array => [
                    'src' => $i->src,
                    'alt' => $i->alt,
                    'title' => $i->title,
                    'dimensions' => $i->dimensions,
                    'image_type' => $i->imageType,
                    'attributes' => $i->attributes,
                ],
                $this->images,
            ),
            'structured_data' => \array_map(
                static fn (StructuredData $s): array => [
                    'data_type' => $s->dataType,
                    'raw_json' => $s->rawJson,
                    'schema_type' => $s->schemaType,
                ],
                $this->structuredData,
            ),
        ];
    }
}
