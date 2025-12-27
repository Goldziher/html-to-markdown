<?php

declare(strict_types=1);

/**
 * Content Filtering Visitor Example
 *
 * This example demonstrates how to use the visitor pattern to filter out
 * unwanted content during HTML-to-Markdown conversion. In this case, we're
 * removing all images and skipping elements with specific classes.
 */

require_once __DIR__ . '/../vendor/autoload.php';

use HtmlToMarkdown\Visitor\AbstractVisitor;
use HtmlToMarkdown\Visitor\NodeContext;
use HtmlToMarkdown\Visitor\VisitResult;
use HtmlToMarkdown\HtmlToMarkdown;

/**
 * Content filter that removes images and ads
 */
class AdAndImageFilter extends AbstractVisitor
{
    /**
     * Skip all images
     */
    public function visitImage(NodeContext $context, string $src, string $alt, ?string $title): array
    {
        return VisitResult::skip();
    }

    /**
     * Skip elements with "ad-" class prefix
     */
    public function visitElementStart(NodeContext $context): array
    {
        $attributes = $context->attributes;

        if (isset($attributes['class'])) {
            $classes = explode(' ', $attributes['class']);
            foreach ($classes as $class) {
                if (str_starts_with($class, 'ad-') || $class === 'advertisement' || $class === 'banner') {
                    return VisitResult::skip();
                }
            }
        }

        return VisitResult::continue();
    }
}

$html = <<<'HTML'
<article>
    <h1>News Article</h1>

    <aside class="ad-sidebar">
        <p>This is an advertisement you won't see in the output.</p>
    </aside>

    <p>Lorem ipsum dolor sit amet...</p>

    <img src="promotional.png" alt="promotional image">

    <p>More content continues here.</p>

    <div class="advertisement">
        <p>Another ad that will be filtered out</p>
    </div>
</article>
HTML;

echo "Default conversion (with ads):\n";
echo HtmlToMarkdown::convert($html);
echo "\n---\n\n";

echo "Filtered conversion (ads removed):\n";
try {
    $filter = new AdAndImageFilter();
    $markdown = HtmlToMarkdown::convertWithVisitor($html, null, $filter);
    echo $markdown;
} catch (Throwable $e) {
    echo "Note: Filter example requires the PHP extension with visitor support\n";
    echo "Error: " . $e->getMessage() . "\n";
}
