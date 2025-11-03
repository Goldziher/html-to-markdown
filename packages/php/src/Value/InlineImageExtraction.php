<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

final readonly class InlineImageExtraction
{
    /**
     * @param list<InlineImage> $inlineImages
     * @param list<InlineImageWarning> $warnings
     */
    public function __construct(
        public string $markdown,
        public array $inlineImages,
        public array $warnings,
    ) {
    }

    /**
     * @param array<string, mixed> $payload
     */
    public static function fromExtensionPayload(array $payload): self
    {
        if (!array_key_exists('markdown', $payload) || !is_string($payload['markdown'])) {
            throw \HtmlToMarkdown\Exception\InvalidOption::because(
                'inline_image_extraction.markdown',
                'extension returned unexpected payload',
            );
        }

        $inlineImagesPayload = $payload['inline_images'] ?? [];
        if (!is_array($inlineImagesPayload)) {
            throw \HtmlToMarkdown\Exception\InvalidOption::because(
                'inline_image_extraction.inline_images',
                'expected list, got ' . get_debug_type($inlineImagesPayload),
            );
        }

        $inlineImages = [];
        foreach ($inlineImagesPayload as $image) {
            if (!is_array($image)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because(
                    'inline_image_extraction.inline_images[]',
                    'expected array, got ' . get_debug_type($image),
                );
            }

            $inlineImages[] = InlineImage::fromExtensionPayload($image);
        }

        $warningsPayload = $payload['warnings'] ?? [];
        if (!is_array($warningsPayload)) {
            throw \HtmlToMarkdown\Exception\InvalidOption::because(
                'inline_image_extraction.warnings',
                'expected list, got ' . get_debug_type($warningsPayload),
            );
        }

        $warnings = [];
        foreach ($warningsPayload as $warning) {
            if (!is_array($warning) || !array_key_exists('index', $warning) || !array_key_exists('message', $warning)) {
                throw \HtmlToMarkdown\Exception\InvalidOption::because(
                    'inline_image_extraction.warnings[]',
                    'expected array{index:int,message:string}',
                );
            }

            $warnings[] = new InlineImageWarning((int) $warning['index'], (string) $warning['message']);
        }

        return new self(
            markdown: (string) $payload['markdown'],
            inlineImages: $inlineImages,
            warnings: $warnings,
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        return [
            'markdown' => $this->markdown,
            'inline_images' => array_map(
                static fn (InlineImage $image): array => [
                    'data' => $image->data,
                    'format' => (string) $image->format,
                    'filename' => $image->filename,
                    'description' => $image->description,
                    'dimensions' => $image->dimensions
                        ? [$image->dimensions->width, $image->dimensions->height]
                        : null,
                    'source' => $image->source->value,
                    'attributes' => $image->attributes,
                ],
                $this->inlineImages,
            ),
            'warnings' => array_map(
                static fn (InlineImageWarning $warning): array => [
                    'index' => $warning->index,
                    'message' => $warning->message,
                ],
                $this->warnings,
            ),
        ];
    }
}
