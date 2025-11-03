<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Enum;

use HtmlToMarkdown\Exception\InvalidOption;

enum NewlineStyle: string
{
    case SPACES = 'spaces';
    case BACKSLASH = 'backslash';

    public static function fromString(string $value): self
    {
        try {
            return self::from($value);
        } catch (\ValueError) {
            throw InvalidOption::forEnum('newline_style', $value, self::cases());
        }
    }
}
