<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Enum;

use HtmlToMarkdown\Exception\InvalidOption;

enum HeadingStyle: string
{
    case UNDERLINED = 'underlined';
    case ATX = 'atx';
    case ATX_CLOSED = 'atx_closed';

    public static function fromString(string $value): self
    {
        try {
            return self::from($value);
        } catch (\ValueError) {
            throw InvalidOption::forEnum('heading_style', $value, self::cases());
        }
    }
}
