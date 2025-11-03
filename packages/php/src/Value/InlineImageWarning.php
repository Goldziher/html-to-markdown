<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Value;

final readonly class InlineImageWarning
{
    public function __construct(
        public int $index,
        public string $message,
    ) {
    }
}
