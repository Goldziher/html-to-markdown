<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Visitor;

use HtmlToMarkdown\Visitor\NodeContext;
use HtmlToMarkdown\Visitor\VisitResult;

/**
 * HtmlVisitor interface for customizing HTML-to-Markdown conversion.
 *
 * Implement this interface to intercept and customize how specific HTML elements
 * are converted to Markdown. You can:
 * - Log element traversal for analytics
 * - Filter out unwanted content
 * - Apply custom transformations
 * - Preserve specific HTML elements as-is
 *
 * All visitor methods are optional. Implement only the ones you need.
 *
 * Example:
 * ```php
 * class MyVisitor implements HtmlVisitor {
 *     public function visitLink(NodeContext $ctx, string $href, string $text, ?string $title): array {
 *         // Log all links
 *         echo "Found link: $href\n";
 *         return VisitResult::continue(); // Use default conversion
 *     }
 *
 *     public function visitImage(NodeContext $ctx, string $src, string $alt, ?string $title): array {
 *         // Skip images
 *         return VisitResult::skip();
 *     }
 * }
 *
 * $visitor = new MyVisitor();
 * $markdown = HtmlToMarkdown::convertWithVisitor($html, null, $visitor);
 * ```
 *
 * @phpstan-import-type VisitResultArray from VisitResult
 */
interface HtmlVisitor
{
    /**
     * Called when an opening tag is encountered.
     *
     * @param NodeContext $context Information about the element
     * @phpstan-return VisitResultArray
     */
    public function visitElementStart(NodeContext $context): array;

    /**
     * Called when a closing tag is encountered.
     *
     * @param NodeContext $context Information about the element
     * @param string $output The generated markdown output for the element's children
     * @phpstan-return VisitResultArray
     */
    public function visitElementEnd(NodeContext $context, string $output): array;

    /**
     * Called for text content.
     *
     * @param NodeContext $context Information about the text node
     * @param string $text The text content
     * @phpstan-return VisitResultArray
     */
    public function visitText(NodeContext $context, string $text): array;

    /**
     * Called for hyperlinks (`<a>` tags).
     *
     * @param NodeContext $context Information about the link element
     * @param string $href The href attribute
     * @param string $text The link text
     * @param string|null $title The title attribute (if present)
     * @phpstan-return VisitResultArray
     */
    public function visitLink(NodeContext $context, string $href, string $text, ?string $title): array;

    /**
     * Called for images (`<img>` tags).
     *
     * @param NodeContext $context Information about the image element
     * @param string $src The src attribute
     * @param string $alt The alt attribute
     * @param string|null $title The title attribute (if present)
     * @phpstan-return VisitResultArray
     */
    public function visitImage(NodeContext $context, string $src, string $alt, ?string $title): array;

    /**
     * Called for headings (`<h1>` through `<h6>`).
     *
     * @param NodeContext $context Information about the heading element
     * @param int $level The heading level (1-6)
     * @param string $text The heading text
     * @param string|null $id The id attribute (if present)
     * @phpstan-return VisitResultArray
     */
    public function visitHeading(NodeContext $context, int $level, string $text, ?string $id): array;

    /**
     * Called for code blocks (`<pre><code>` or with language hint).
     *
     * @param NodeContext $context Information about the code block
     * @param string|null $lang The language hint (from class attribute)
     * @param string $code The code content
     * @phpstan-return VisitResultArray
     */
    public function visitCodeBlock(NodeContext $context, ?string $lang, string $code): array;

    /**
     * Called for inline code (`<code>`).
     *
     * @param NodeContext $context Information about the code element
     * @param string $code The code content
     * @phpstan-return VisitResultArray
     */
    public function visitCodeInline(NodeContext $context, string $code): array;

    /**
     * Called for list items (`<li>`).
     *
     * @param NodeContext $context Information about the list item
     * @param bool $ordered Whether the list is ordered (true) or unordered (false)
     * @param string $marker The list marker (-, *, +, or 1., 2., etc.)
     * @param string $text The list item text
     * @phpstan-return VisitResultArray
     */
    public function visitListItem(NodeContext $context, bool $ordered, string $marker, string $text): array;

    /**
     * Called before processing a list (`<ul>` or `<ol>`).
     *
     * @param NodeContext $context Information about the list element
     * @param bool $ordered Whether the list is ordered (true) or unordered (false)
     * @phpstan-return VisitResultArray
     */
    public function visitListStart(NodeContext $context, bool $ordered): array;

    /**
     * Called after processing a list.
     *
     * @param NodeContext $context Information about the list element
     * @param bool $ordered Whether the list is ordered (true) or unordered (false)
     * @param string $output The generated markdown for the list
     * @phpstan-return VisitResultArray
     */
    public function visitListEnd(NodeContext $context, bool $ordered, string $output): array;

    /**
     * Called before processing a table.
     *
     * @param NodeContext $context Information about the table element
     * @phpstan-return VisitResultArray
     */
    public function visitTableStart(NodeContext $context): array;

    /**
     * Called for each table row.
     *
     * @param NodeContext $context Information about the table row
     * @param string[] $cells The cell contents (including header cells)
     * @param bool $isHeader Whether this is a header row
     * @phpstan-return VisitResultArray
     */
    public function visitTableRow(NodeContext $context, array $cells, bool $isHeader): array;

    /**
     * Called after processing a table.
     *
     * @param NodeContext $context Information about the table element
     * @param string $output The generated markdown for the table
     * @phpstan-return VisitResultArray
     */
    public function visitTableEnd(NodeContext $context, string $output): array;

    /**
     * Called for blockquotes (`<blockquote>`).
     *
     * @param NodeContext $context Information about the blockquote element
     * @param string $content The blockquote content
     * @param int $depth The nesting depth
     * @phpstan-return VisitResultArray
     */
    public function visitBlockquote(NodeContext $context, string $content, int $depth): array;

    /**
     * Called for strong/bold text (`<strong>`, `<b>`).
     *
     * @param NodeContext $context Information about the element
     * @param string $text The text content
     * @phpstan-return VisitResultArray
     */
    public function visitStrong(NodeContext $context, string $text): array;

    /**
     * Called for emphasis/italic text (`<em>`, `<i>`).
     *
     * @param NodeContext $context Information about the element
     * @param string $text The text content
     * @phpstan-return VisitResultArray
     */
    public function visitEmphasis(NodeContext $context, string $text): array;

    /**
     * Called for strikethrough text (`<strike>`, `<s>`, `<del>`).
     *
     * @param NodeContext $context Information about the element
     * @param string $text The text content
     * @phpstan-return VisitResultArray
     */
    public function visitStrikethrough(NodeContext $context, string $text): array;

    /**
     * Called for underlined text (`<u>`).
     *
     * @param NodeContext $context Information about the element
     * @param string $text The text content
     * @phpstan-return VisitResultArray
     */
    public function visitUnderline(NodeContext $context, string $text): array;

    /**
     * Called for subscript text (`<sub>`).
     *
     * @param NodeContext $context Information about the element
     * @param string $text The text content
     * @phpstan-return VisitResultArray
     */
    public function visitSubscript(NodeContext $context, string $text): array;

    /**
     * Called for superscript text (`<sup>`).
     *
     * @param NodeContext $context Information about the element
     * @param string $text The text content
     * @phpstan-return VisitResultArray
     */
    public function visitSuperscript(NodeContext $context, string $text): array;

    /**
     * Called for highlighted/marked text (`<mark>`).
     *
     * @param NodeContext $context Information about the element
     * @param string $text The text content
     * @phpstan-return VisitResultArray
     */
    public function visitMark(NodeContext $context, string $text): array;

    /**
     * Called for line breaks (`<br>`).
     *
     * @param NodeContext $context Information about the element
     * @phpstan-return VisitResultArray
     */
    public function visitLineBreak(NodeContext $context): array;

    /**
     * Called for horizontal rules (`<hr>`).
     *
     * @param NodeContext $context Information about the element
     * @phpstan-return VisitResultArray
     */
    public function visitHorizontalRule(NodeContext $context): array;

    /**
     * Called for custom/unknown elements.
     *
     * @param NodeContext $context Information about the element
     * @param string $tagName The element tag name
     * @param string $html The raw HTML of the element
     * @phpstan-return VisitResultArray
     */
    public function visitCustomElement(NodeContext $context, string $tagName, string $html): array;

    /**
     * Called before processing a definition list (`<dl>`).
     *
     * @param NodeContext $context Information about the definition list element
     * @phpstan-return VisitResultArray
     */
    public function visitDefinitionListStart(NodeContext $context): array;

    /**
     * Called for definition terms (`<dt>`).
     *
     * @param NodeContext $context Information about the definition term
     * @param string $text The term text
     * @phpstan-return VisitResultArray
     */
    public function visitDefinitionTerm(NodeContext $context, string $text): array;

    /**
     * Called for definition descriptions (`<dd>`).
     *
     * @param NodeContext $context Information about the definition description
     * @param string $text The description text
     * @phpstan-return VisitResultArray
     */
    public function visitDefinitionDescription(NodeContext $context, string $text): array;

    /**
     * Called after processing a definition list.
     *
     * @param NodeContext $context Information about the definition list element
     * @param string $output The generated markdown for the definition list
     * @phpstan-return VisitResultArray
     */
    public function visitDefinitionListEnd(NodeContext $context, string $output): array;

    /**
     * Called for forms (`<form>`).
     *
     * @param NodeContext $context Information about the form element
     * @param string|null $action The form action attribute
     * @param string|null $method The form method attribute
     * @phpstan-return VisitResultArray
     */
    public function visitForm(NodeContext $context, ?string $action, ?string $method): array;

    /**
     * Called for form inputs (`<input>`).
     *
     * @param NodeContext $context Information about the input element
     * @param string $inputType The input type attribute
     * @param string|null $name The input name attribute
     * @param string|null $value The input value attribute
     * @phpstan-return VisitResultArray
     */
    public function visitInput(NodeContext $context, string $inputType, ?string $name, ?string $value): array;

    /**
     * Called for buttons (`<button>`).
     *
     * @param NodeContext $context Information about the button element
     * @param string $text The button text
     * @phpstan-return VisitResultArray
     */
    public function visitButton(NodeContext $context, string $text): array;

    /**
     * Called for audio elements (`<audio>`).
     *
     * @param NodeContext $context Information about the audio element
     * @param string|null $src The src attribute
     * @phpstan-return VisitResultArray
     */
    public function visitAudio(NodeContext $context, ?string $src): array;

    /**
     * Called for video elements (`<video>`).
     *
     * @param NodeContext $context Information about the video element
     * @param string|null $src The src attribute
     * @phpstan-return VisitResultArray
     */
    public function visitVideo(NodeContext $context, ?string $src): array;

    /**
     * Called for iframe elements (`<iframe>`).
     *
     * @param NodeContext $context Information about the iframe element
     * @param string|null $src The src attribute
     * @phpstan-return VisitResultArray
     */
    public function visitIframe(NodeContext $context, ?string $src): array;

    /**
     * Called for details/disclosure elements (`<details>`).
     *
     * @param NodeContext $context Information about the details element
     * @param bool $open Whether the details element is open
     * @phpstan-return VisitResultArray
     */
    public function visitDetails(NodeContext $context, bool $open): array;

    /**
     * Called for summary elements (`<summary>`).
     *
     * @param NodeContext $context Information about the summary element
     * @param string $text The summary text
     * @phpstan-return VisitResultArray
     */
    public function visitSummary(NodeContext $context, string $text): array;

    /**
     * Called before processing a figure element (`<figure>`).
     *
     * @param NodeContext $context Information about the figure element
     * @phpstan-return VisitResultArray
     */
    public function visitFigureStart(NodeContext $context): array;

    /**
     * Called for figure captions (`<figcaption>`).
     *
     * @param NodeContext $context Information about the figcaption element
     * @param string $text The caption text
     * @phpstan-return VisitResultArray
     */
    public function visitFigcaption(NodeContext $context, string $text): array;

    /**
     * Called after processing a figure element.
     *
     * @param NodeContext $context Information about the figure element
     * @param string $output The generated markdown for the figure
     * @phpstan-return VisitResultArray
     */
    public function visitFigureEnd(NodeContext $context, string $output): array;
}
