<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Enum;

use HtmlToMarkdown\Exception\InvalidOption;

enum HighlightStyle: string
{
    case DOUBLE_EQUAL = 'double_equal';
    case HTML = 'html';
    case BOLD = 'bold';
    case NONE = 'none';

    public static function fromString(string $value): self
    {
        try {
            return self::from($value);
        } catch (\ValueError) {
            throw InvalidOption::forEnum('highlight_style', $value, self::cases());
        }
    }
}
