<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Exception;

use BackedEnum;
use InvalidArgumentException;

final class InvalidOption extends InvalidArgumentException
{
    /**
     * @param list<BackedEnum> $cases
     */
    public static function forEnum(string $option, string $value, array $cases): self
    {
        $allowed = \array_map(
            static fn (BackedEnum $case): string => (string) $case->value,
            $cases,
        );

        return new self(\sprintf(
            "Invalid value '%s' for option '%s'. Allowed values: %s",
            $value,
            $option,
            \implode(', ', $allowed),
        ));
    }

    public static function because(string $option, string $message): self
    {
        return new self(\sprintf("Invalid value for option '%s': %s", $option, $message));
    }
}
