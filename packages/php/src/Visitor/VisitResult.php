<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Visitor;

/**
 * VisitResult is returned by visitor methods to control the conversion behavior.
 *
 * Each visitor method can return one of five result types:
 * - Continue: Use the default markdown conversion
 * - Skip: Skip this element entirely (don't convert to markdown)
 * - PreserveHtml: Keep the element as raw HTML in the output
 * - Custom: Replace with custom markdown output
 * - Error: Stop conversion and report an error
 *
 * @phpstan-type VisitResultArray array{
 *     type: 'continue'|'skip'|'preserve_html'|'custom'|'error',
 *     output?: string,
 *     message?: string,
 * }
 */
final class VisitResult
{
    /**
     * Use the default markdown conversion for this element.
     *
     * @phpstan-return VisitResultArray
     */
    public static function continue(): array
    {
        return ['type' => 'continue'];
    }

    /**
     * Skip this element entirely (don't convert to markdown).
     *
     * @phpstan-return VisitResultArray
     */
    public static function skip(): array
    {
        return ['type' => 'skip'];
    }

    /**
     * Keep the element as raw HTML in the output.
     *
     * @phpstan-return VisitResultArray
     */
    public static function preserveHtml(): array
    {
        return ['type' => 'preserve_html'];
    }

    /**
     * Replace with custom markdown output.
     *
     * @param string $output The custom markdown to use instead
     * @phpstan-return VisitResultArray
     */
    public static function custom(string $output): array
    {
        return [
            'type' => 'custom',
            'output' => $output,
        ];
    }

    /**
     * Stop conversion and report an error.
     *
     * @param string $message The error message
     * @phpstan-return VisitResultArray
     */
    public static function error(string $message): array
    {
        return [
            'type' => 'error',
            'message' => $message,
        ];
    }
}
