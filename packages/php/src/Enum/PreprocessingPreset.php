<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Enum;

use HtmlToMarkdown\Exception\InvalidOption;

enum PreprocessingPreset: string
{
    case MINIMAL = 'minimal';
    case STANDARD = 'standard';
    case AGGRESSIVE = 'aggressive';

    public static function fromString(string $value): self
    {
        try {
            return self::from($value);
        } catch (\ValueError) {
            throw InvalidOption::forEnum('preprocessing.preset', $value, self::cases());
        }
    }
}
