<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Enum;

use HtmlToMarkdown\Exception\InvalidOption;

enum InlineImageSource: string
{
    case IMG_DATA_URI = 'img_data_uri';
    case SVG_ELEMENT = 'svg_element';

    public static function fromString(string $value): self
    {
        try {
            return self::from($value);
        } catch (\ValueError) {
            throw InvalidOption::forEnum('inline_image.source', $value, self::cases());
        }
    }
}
