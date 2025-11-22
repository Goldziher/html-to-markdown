<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Internal;

use HtmlToMarkdown\Exception\InvalidOption;

/**
 * @internal
 */
final class TypeAssertions
{
    private function __construct()
    {
    }

    public static function bool(mixed $value, string $option): bool
    {
        if (\is_bool($value)) {
            return $value;
        }

        throw InvalidOption::because($option, \sprintf('expected bool, got %s', \get_debug_type($value)));
    }

    public static function string(mixed $value, string $option): string
    {
        if (\is_string($value)) {
            return $value;
        }

        throw InvalidOption::because($option, \sprintf('expected string, got %s', \get_debug_type($value)));
    }

    public static function stringOrNull(mixed $value, string $option): ?string
    {
        if ($value === null) {
            return null;
        }

        return self::string($value, $option);
    }

    public static function positiveInt(mixed $value, string $option): int
    {
        if (\is_int($value) && $value >= 0) {
            return $value;
        }

        throw InvalidOption::because($option, \sprintf('expected non-negative integer, got %s', \get_debug_type($value)));
    }

    /**
     * @return list<string>
     */
    public static function stringList(mixed $value, string $option): array
    {
        if (!\is_array($value)) {
            throw InvalidOption::because($option, \sprintf('expected array of strings, got %s', \get_debug_type($value)));
        }

        $result = [];
        foreach ($value as $index => $entry) {
            if (!\is_string($entry)) {
                throw InvalidOption::because(
                    \sprintf('%s[%s]', $option, (string) $index),
                    \sprintf('expected string, got %s', \get_debug_type($entry)),
                );
            }

            $result[] = $entry;
        }

        return \array_values($result);
    }
}
