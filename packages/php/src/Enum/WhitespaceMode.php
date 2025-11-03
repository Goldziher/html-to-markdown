<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Enum;

use HtmlToMarkdown\Exception\InvalidOption;

enum WhitespaceMode: string
{
    case NORMALIZED = 'normalized';
    case STRICT = 'strict';

    public static function fromString(string $value): self
    {
        try {
            return self::from($value);
        } catch (\ValueError) {
            throw InvalidOption::forEnum('whitespace_mode', $value, self::cases());
        }
    }
}
