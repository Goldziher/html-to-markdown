<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

final class InlineImageFormat
{
    private const KNOWN_FORMATS = [
        'png',
        'jpeg',
        'gif',
        'bmp',
        'webp',
        'svg',
    ];

    private function __construct(
        public readonly string $value,
    ) {
    }

    public static function fromString(string $value): self
    {
        return new self($value);
    }

    public function isKnown(): bool
    {
        return \in_array($this->value, self::KNOWN_FORMATS, true);
    }

    public function equals(self $other): bool
    {
        return $this->value === $other->value;
    }

    public function __toString(): string
    {
        return $this->value;
    }
}
