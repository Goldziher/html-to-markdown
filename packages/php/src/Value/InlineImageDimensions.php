<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

use HtmlToMarkdown\Exception\InvalidOption;

final readonly class InlineImageDimensions
{
    public function __construct(
        public int $width,
        public int $height,
    ) {
        if ($width <= 0 || $height <= 0) {
            throw InvalidOption::because('inline_image.dimensions', 'width and height must be positive integers');
        }
    }

    /**
     * @param array{0:int,1:int} $dimensions
     */
    public static function fromArray(array $dimensions): self
    {
        return new self($dimensions[0], $dimensions[1]);
    }
}
