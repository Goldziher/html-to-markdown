<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Visitor;

/**
 * AbstractVisitor provides a convenient base class for creating custom visitors.
 *
 * All visitor methods return VisitResult::continue() by default,
 * allowing you to override only the methods you need to customize.
 *
 * Example:
 * ```php
 * class Analytics extends AbstractVisitor {
 *     public function __construct(
 *         private array $links = [],
 *         private array $images = [],
 *     ) {
 *     }
 *
 *     public function visitLink(NodeContext $ctx, string $href, string $text, ?string $title): array {
 *         $this->links[] = ['href' => $href, 'text' => $text];
 *         return VisitResult::continue();
 *     }
 *
 *     public function visitImage(NodeContext $ctx, string $src, string $alt, ?string $title): array {
 *         $this->images[] = ['src' => $src, 'alt' => $alt];
 *         return VisitResult::continue();
 *     }
 *
 *     public function getStats(): array {
 *         return ['links' => count($this->links), 'images' => count($this->images)];
 *     }
 * }
 * ```
 *
 * @phpstan-import-type VisitResultArray from VisitResult
 */
abstract class AbstractVisitor implements HtmlVisitor
{
    public function visitElementStart(NodeContext $context): array
    {
        return VisitResult::continue();
    }

    public function visitElementEnd(NodeContext $context, string $output): array
    {
        return VisitResult::continue();
    }

    public function visitText(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitLink(NodeContext $context, string $href, string $text, ?string $title): array
    {
        return VisitResult::continue();
    }

    public function visitImage(NodeContext $context, string $src, string $alt, ?string $title): array
    {
        return VisitResult::continue();
    }

    public function visitHeading(NodeContext $context, int $level, string $text, ?string $id): array
    {
        return VisitResult::continue();
    }

    public function visitCodeBlock(NodeContext $context, ?string $lang, string $code): array
    {
        return VisitResult::continue();
    }

    public function visitCodeInline(NodeContext $context, string $code): array
    {
        return VisitResult::continue();
    }

    public function visitListItem(NodeContext $context, bool $ordered, string $marker, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitListStart(NodeContext $context, bool $ordered): array
    {
        return VisitResult::continue();
    }

    public function visitListEnd(NodeContext $context, bool $ordered, string $output): array
    {
        return VisitResult::continue();
    }

    public function visitTableStart(NodeContext $context): array
    {
        return VisitResult::continue();
    }

    public function visitTableRow(NodeContext $context, array $cells, bool $isHeader): array
    {
        return VisitResult::continue();
    }

    public function visitTableEnd(NodeContext $context, string $output): array
    {
        return VisitResult::continue();
    }

    public function visitBlockquote(NodeContext $context, string $content, int $depth): array
    {
        return VisitResult::continue();
    }

    public function visitStrong(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitEmphasis(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitStrikethrough(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitUnderline(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitSubscript(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitSuperscript(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitMark(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitLineBreak(NodeContext $context): array
    {
        return VisitResult::continue();
    }

    public function visitHorizontalRule(NodeContext $context): array
    {
        return VisitResult::continue();
    }

    public function visitCustomElement(NodeContext $context, string $tagName, string $html): array
    {
        return VisitResult::continue();
    }

    public function visitDefinitionListStart(NodeContext $context): array
    {
        return VisitResult::continue();
    }

    public function visitDefinitionTerm(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitDefinitionDescription(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitDefinitionListEnd(NodeContext $context, string $output): array
    {
        return VisitResult::continue();
    }

    public function visitForm(NodeContext $context, ?string $action, ?string $method): array
    {
        return VisitResult::continue();
    }

    public function visitInput(NodeContext $context, string $inputType, ?string $name, ?string $value): array
    {
        return VisitResult::continue();
    }

    public function visitButton(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitAudio(NodeContext $context, ?string $src): array
    {
        return VisitResult::continue();
    }

    public function visitVideo(NodeContext $context, ?string $src): array
    {
        return VisitResult::continue();
    }

    public function visitIframe(NodeContext $context, ?string $src): array
    {
        return VisitResult::continue();
    }

    public function visitDetails(NodeContext $context, bool $open): array
    {
        return VisitResult::continue();
    }

    public function visitSummary(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitFigureStart(NodeContext $context): array
    {
        return VisitResult::continue();
    }

    public function visitFigcaption(NodeContext $context, string $text): array
    {
        return VisitResult::continue();
    }

    public function visitFigureEnd(NodeContext $context, string $output): array
    {
        return VisitResult::continue();
    }
}
