<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Exception;

use RuntimeException;

final class ConversionFailed extends RuntimeException
{
    public static function withMessage(string $message): self
    {
        return new self(\sprintf('Conversion failed: %s', $message));
    }
}
