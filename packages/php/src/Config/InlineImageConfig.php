<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Config;

use HtmlToMarkdown\Exception\InvalidOption;
use HtmlToMarkdown\Internal\TypeAssertions;

final readonly class InlineImageConfig
{
    public function __construct(
        public int $maxDecodedSizeBytes = 5 * 1024 * 1024,
        public ?string $filenamePrefix = null,
        public bool $captureSvg = true,
        public bool $inferDimensions = false,
    ) {
        if ($this->maxDecodedSizeBytes <= 0) {
            throw InvalidOption::because(
                'inline_image_config.max_decoded_size_bytes',
                'must be greater than zero',
            );
        }
    }

    /**
     * @param array<string, mixed> $input
     */
    public static function fromArray(array $input): self
    {
        $defaults = new self();

        return new self(
            maxDecodedSizeBytes: \array_key_exists('max_decoded_size_bytes', $input)
                ? TypeAssertions::positiveInt($input['max_decoded_size_bytes'], 'inline_image_config.max_decoded_size_bytes')
                : $defaults->maxDecodedSizeBytes,
            filenamePrefix: \array_key_exists('filename_prefix', $input)
                ? TypeAssertions::stringOrNull($input['filename_prefix'], 'inline_image_config.filename_prefix')
                : $defaults->filenamePrefix,
            captureSvg: \array_key_exists('capture_svg', $input)
                ? TypeAssertions::bool($input['capture_svg'], 'inline_image_config.capture_svg')
                : $defaults->captureSvg,
            inferDimensions: \array_key_exists('infer_dimensions', $input)
                ? TypeAssertions::bool($input['infer_dimensions'], 'inline_image_config.infer_dimensions')
                : $defaults->inferDimensions,
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        return [
            'max_decoded_size_bytes' => $this->maxDecodedSizeBytes,
            'filename_prefix' => $this->filenamePrefix,
            'capture_svg' => $this->captureSvg,
            'infer_dimensions' => $this->inferDimensions,
        ];
    }
}
