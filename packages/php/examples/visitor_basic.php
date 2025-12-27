<?php

declare(strict_types=1);

/**
 * Basic Visitor Example
 *
 * This example demonstrates how to use the visitor pattern to customize
 * HTML-to-Markdown conversion by intercepting and modifying how certain
 * elements are processed.
 */

require_once __DIR__ . '/../vendor/autoload.php';

use HtmlToMarkdown\Visitor\AbstractVisitor;
use HtmlToMarkdown\Visitor\NodeContext;
use HtmlToMarkdown\Visitor\VisitResult;
use HtmlToMarkdown\HtmlToMarkdown;

/**
 * Custom visitor that wraps all links in extra brackets for emphasis
 */
class LinkHighlighter extends AbstractVisitor
{
    public function visitLink(NodeContext $context, string $href, string $text, ?string $title): array
    {
        return VisitResult::custom(">> [{$text}]({$href}) <<");
    }
}

$html = <<<'HTML'
<article>
    <h1>Welcome to My Blog</h1>
    <p>Check out <a href="https://example.com">this example</a> and <a href="https://test.com">this one too</a>.</p>
</article>
HTML;

echo "Default conversion:\n";
echo HtmlToMarkdown::convert($html);
echo "\n---\n\n";

echo "With custom visitor (if supported):\n";
try {
    $visitor = new LinkHighlighter();
    $markdown = HtmlToMarkdown::convertWithVisitor($html, null, $visitor);
    echo $markdown;
} catch (Throwable $e) {
    echo "Note: Visitor example requires the PHP extension with visitor support\n";
    echo "Error: " . $e->getMessage() . "\n";
}
