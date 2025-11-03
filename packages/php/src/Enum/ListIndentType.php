<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Enum;

use HtmlToMarkdown\Exception\InvalidOption;

enum ListIndentType: string
{
    case SPACES = 'spaces';
    case TABS = 'tabs';

    public static function fromString(string $value): self
    {
        try {
            return self::from($value);
        } catch (\ValueError) {
            throw InvalidOption::forEnum('list_indent_type', $value, self::cases());
        }
    }
}
