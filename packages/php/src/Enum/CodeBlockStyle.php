<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Enum;

use HtmlToMarkdown\Exception\InvalidOption;

enum CodeBlockStyle: string
{
    case INDENTED = 'indented';
    case BACKTICKS = 'backticks';
    case TILDES = 'tildes';

    public static function fromString(string $value): self
    {
        try {
            return self::from($value);
        } catch (\ValueError) {
            throw InvalidOption::forEnum('code_block_style', $value, self::cases());
        }
    }
}
