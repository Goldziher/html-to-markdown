<?php

declare(strict_types=1);

/**
 * Analytics and Statistics Visitor Example
 *
 * This example demonstrates how to use the visitor pattern to gather
 * statistics about HTML content during conversion. This is useful for
 * analytics, reporting, or content validation.
 */

require_once __DIR__ . '/../vendor/autoload.php';

use HtmlToMarkdown\Visitor\AbstractVisitor;
use HtmlToMarkdown\Visitor\NodeContext;
use HtmlToMarkdown\Visitor\VisitResult;
use HtmlToMarkdown\HtmlToMarkdown;

/**
 * Analytics visitor that collects content statistics
 */
class ContentAnalytics extends AbstractVisitor
{
    public int $linkCount = 0;
    public int $imageCount = 0;
    public int $headingCount = 0;
    public int $codeBlockCount = 0;
    public int $tableCount = 0;

    /** @var array<string, int> */
    public array $headingsByLevel = [1 => 0, 2 => 0, 3 => 0, 4 => 0, 5 => 0, 6 => 0];

    /** @var array<string> */
    public array $links = [];

    /** @var array<string> */
    public array $images = [];

    /**
     * Track links
     */
    public function visitLink(NodeContext $context, string $href, string $text, ?string $title): array
    {
        $this->linkCount++;
        $this->links[] = [
            'href' => $href,
            'text' => $text,
            'title' => $title,
        ];

        return VisitResult::continue();
    }

    /**
     * Track images
     */
    public function visitImage(NodeContext $context, string $src, string $alt, ?string $title): array
    {
        $this->imageCount++;
        $this->images[] = [
            'src' => $src,
            'alt' => $alt,
        ];

        return VisitResult::continue();
    }

    /**
     * Track headings and their levels
     */
    public function visitHeading(NodeContext $context, int $level, string $text, ?string $id): array
    {
        $this->headingCount++;
        if (isset($this->headingsByLevel[$level])) {
            $this->headingsByLevel[$level]++;
        }

        return VisitResult::continue();
    }

    /**
     * Track code blocks
     */
    public function visitCodeBlock(NodeContext $context, ?string $lang, string $code): array
    {
        $this->codeBlockCount++;
        return VisitResult::continue();
    }

    /**
     * Track tables
     */
    public function visitTableStart(NodeContext $context): array
    {
        $this->tableCount++;
        return VisitResult::continue();
    }

    /**
     * Generate a statistics report
     */
    public function getReport(): string
    {
        $report = "Content Statistics Report\n";
        $report .= "==========================\n\n";

        $report .= "Overall Metrics:\n";
        $report .= sprintf("- Links: %d\n", $this->linkCount);
        $report .= sprintf("- Images: %d\n", $this->imageCount);
        $report .= sprintf("- Headings: %d\n", $this->headingCount);
        $report .= sprintf("- Code Blocks: %d\n", $this->codeBlockCount);
        $report .= sprintf("- Tables: %d\n\n", $this->tableCount);

        if ($this->headingCount > 0) {
            $report .= "Heading Breakdown:\n";
            foreach ($this->headingsByLevel as $level => $count) {
                if ($count > 0) {
                    $report .= sprintf("- H%d: %d\n", $level, $count);
                }
            }
            $report .= "\n";
        }

        if (!empty($this->links)) {
            $report .= sprintf("Links (%d):\n", count($this->links));
            foreach ($this->links as $i => $link) {
                $report .= sprintf("%d. [%s](%s)\n", $i + 1, $link['text'] ?? '(no text)', $link['href']);
            }
            $report .= "\n";
        }

        if (!empty($this->images)) {
            $report .= sprintf("Images (%d):\n", count($this->images));
            foreach ($this->images as $i => $image) {
                $report .= sprintf("%d. %s - alt: \"%s\"\n", $i + 1, $image['src'], $image['alt'] ?? '');
            }
        }

        return $report;
    }
}

// Example HTML with various content types
$html = <<<'HTML'
<article>
    <h1>Complete Guide to Web Development</h1>

    <h2>Frontend Technologies</h2>
    <p>Learn about <a href="/html">HTML</a>, <a href="/css">CSS</a>, and <a href="/js">JavaScript</a>.</p>

    <h3>HTML Basics</h3>
    <p>Here's a simple example:</p>
    <pre><code class="language-html">&lt;div&gt;Hello World&lt;/div&gt;</code></pre>

    <h2>Resources</h2>
    <table>
        <tr>
            <th>Resource</th>
            <th>URL</th>
        </tr>
        <tr>
            <td>MDN</td>
            <td>https://developer.mozilla.org</td>
        </tr>
    </table>

    <p>Check out <img src="diagram.png" alt="Architecture diagram"></p>

    <h3>Code Example</h3>
    <pre><code class="language-php">echo "Hello";</code></pre>
</article>
HTML;

// Create analytics visitor
$analytics = new ContentAnalytics();

// Convert with analytics
try {
    $markdown = HtmlToMarkdown::convertWithVisitor($html, null, $analytics);

    echo "Converted Markdown:\n";
    echo "==================\n";
    echo $markdown;
    echo "\n\n";

    // Display statistics
    echo $analytics->getReport();
} catch (Throwable $e) {
    echo "Note: Analytics example requires the PHP extension with visitor support\n";
    echo "Error: " . $e->getMessage() . "\n";

    // Still show what the report would look like after a normal conversion
    echo "\nExample statistics (if extension was available):\n";
    HtmlToMarkdown::convert($html);
}
