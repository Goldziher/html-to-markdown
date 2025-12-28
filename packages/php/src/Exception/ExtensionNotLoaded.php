<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Exception;

use RuntimeException;

final class ExtensionNotLoaded extends RuntimeException
{
    public static function create(): self
    {
        return new self(
            'The html_to_markdown extension is not loaded. Install it via PIE (goldziher/html-to-markdown)'
            . ' or provide the compiled binary.'
        );
    }
}
