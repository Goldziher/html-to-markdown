<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Tests;

use HtmlToMarkdown\Visitor\AbstractVisitor;
use HtmlToMarkdown\Visitor\HtmlVisitor;
use HtmlToMarkdown\Visitor\NodeContext;
use HtmlToMarkdown\Visitor\VisitResult;
use PHPUnit\Framework\TestCase;

final class VisitorTest extends TestCase
{
    public function testNodeContextCreationFromArray(): void
    {
        $data = [
            'node_type' => 'element',
            'tag_name' => 'p',
            'attributes' => ['class' => 'highlight'],
            'depth' => 1,
            'index_in_parent' => 0,
            'parent_tag' => 'div',
            'is_inline' => false,
        ];

        $context = NodeContext::fromArray($data);

        self::assertSame('element', $context->nodeType);
        self::assertSame('p', $context->tagName);
        self::assertSame(['class' => 'highlight'], $context->attributes);
        self::assertSame(1, $context->depth);
        self::assertSame(0, $context->indexInParent);
        self::assertSame('div', $context->parentTag);
        self::assertFalse($context->isInline);
    }

    public function testNodeContextWithNullParentTag(): void
    {
        $data = [
            'node_type' => 'element',
            'tag_name' => 'div',
            'attributes' => [],
            'depth' => 0,
            'index_in_parent' => 0,
            'parent_tag' => null,
            'is_inline' => false,
        ];

        $context = NodeContext::fromArray($data);

        self::assertNull($context->parentTag);
    }

    public function testNodeContextConvertToArray(): void
    {
        $context = new NodeContext(
            nodeType: 'element',
            tagName: 'a',
            attributes: ['href' => 'https://example.com'],
            depth: 2,
            indexInParent: 1,
            parentTag: 'p',
            isInline: true,
        );

        $array = $context->toArray();

        self::assertSame('element', $array['node_type']);
        self::assertSame('a', $array['tag_name']);
        self::assertSame(['href' => 'https://example.com'], $array['attributes']);
        self::assertSame(2, $array['depth']);
        self::assertSame(1, $array['index_in_parent']);
        self::assertSame('p', $array['parent_tag']);
        self::assertTrue($array['is_inline']);
    }

    public function testVisitResultContinue(): void
    {
        $result = VisitResult::continue();

        self::assertSame('continue', $result['type']);
        self::assertFalse(isset($result['output']));
    }

    public function testVisitResultSkip(): void
    {
        $result = VisitResult::skip();

        self::assertSame('skip', $result['type']);
    }

    public function testVisitResultPreserveHtml(): void
    {
        $result = VisitResult::preserveHtml();

        self::assertSame('preserve_html', $result['type']);
    }

    public function testVisitResultCustom(): void
    {
        $result = VisitResult::custom('**bold**');

        self::assertSame('custom', $result['type']);
        self::assertSame('**bold**', $result['output'] ?? '');
    }

    public function testVisitResultError(): void
    {
        $result = VisitResult::error('Invalid element');

        self::assertSame('error', $result['type']);
        self::assertSame('Invalid element', $result['message'] ?? '');
    }

    public function testAbstractVisitorImplementsAllMethods(): void
    {
        $visitor = new class () extends AbstractVisitor {
            /** @var array<int, array<int|string, mixed>> */
            public array $calls = [];

            public function visitText(NodeContext $context, string $text): array
            {
                $this->calls[] = ['visitText', $text];
                return VisitResult::continue();
            }
        };

        $ctx = new NodeContext('text', '', [], 0, 0, null, false);

        self::assertSame(VisitResult::continue(), $visitor->visitElementStart($ctx));
        self::assertSame(VisitResult::continue(), $visitor->visitElementEnd($ctx, ''));
        self::assertSame(VisitResult::continue(), $visitor->visitLink($ctx, 'http://test', 'test', null));
        self::assertSame(VisitResult::continue(), $visitor->visitImage($ctx, 'img.png', 'alt', null));
        self::assertSame(VisitResult::continue(), $visitor->visitHeading($ctx, 1, 'title', null));
        self::assertSame(VisitResult::continue(), $visitor->visitCodeBlock($ctx, 'php', 'echo "hi";'));
        self::assertSame(VisitResult::continue(), $visitor->visitCodeInline($ctx, 'code'));
        self::assertSame(VisitResult::continue(), $visitor->visitListItem($ctx, false, '-', 'item'));
        self::assertSame(VisitResult::continue(), $visitor->visitListStart($ctx, false));
        self::assertSame(VisitResult::continue(), $visitor->visitListEnd($ctx, false, ''));
        self::assertSame(VisitResult::continue(), $visitor->visitTableStart($ctx));
        self::assertSame(VisitResult::continue(), $visitor->visitTableRow($ctx, ['col1', 'col2'], false));
        self::assertSame(VisitResult::continue(), $visitor->visitTableEnd($ctx, ''));
        self::assertSame(VisitResult::continue(), $visitor->visitBlockquote($ctx, 'quoted', 1));
        self::assertSame(VisitResult::continue(), $visitor->visitStrong($ctx, 'bold'));
        self::assertSame(VisitResult::continue(), $visitor->visitEmphasis($ctx, 'italic'));
        self::assertSame(VisitResult::continue(), $visitor->visitStrikethrough($ctx, 'struck'));
        self::assertSame(VisitResult::continue(), $visitor->visitUnderline($ctx, 'underlined'));
        self::assertSame(VisitResult::continue(), $visitor->visitSubscript($ctx, 'sub'));
        self::assertSame(VisitResult::continue(), $visitor->visitSuperscript($ctx, 'sup'));
        self::assertSame(VisitResult::continue(), $visitor->visitMark($ctx, 'marked'));
        self::assertSame(VisitResult::continue(), $visitor->visitLineBreak($ctx));
        self::assertSame(VisitResult::continue(), $visitor->visitHorizontalRule($ctx));
        self::assertSame(VisitResult::continue(), $visitor->visitCustomElement($ctx, 'custom', '<custom></custom>'));
        self::assertSame(VisitResult::continue(), $visitor->visitDefinitionListStart($ctx));
        self::assertSame(VisitResult::continue(), $visitor->visitDefinitionTerm($ctx, 'term'));
        self::assertSame(VisitResult::continue(), $visitor->visitDefinitionDescription($ctx, 'desc'));
        self::assertSame(VisitResult::continue(), $visitor->visitDefinitionListEnd($ctx, ''));
        self::assertSame(VisitResult::continue(), $visitor->visitForm($ctx, 'POST', '/submit'));
        self::assertSame(VisitResult::continue(), $visitor->visitInput($ctx, 'text', 'name', 'value'));
        self::assertSame(VisitResult::continue(), $visitor->visitButton($ctx, 'Click'));
        self::assertSame(VisitResult::continue(), $visitor->visitAudio($ctx, 'audio.mp3'));
        self::assertSame(VisitResult::continue(), $visitor->visitVideo($ctx, 'video.mp4'));
        self::assertSame(VisitResult::continue(), $visitor->visitIframe($ctx, 'https://example.com'));
        self::assertSame(VisitResult::continue(), $visitor->visitDetails($ctx, true));
        self::assertSame(VisitResult::continue(), $visitor->visitSummary($ctx, 'summary'));
        self::assertSame(VisitResult::continue(), $visitor->visitFigureStart($ctx));
        self::assertSame(VisitResult::continue(), $visitor->visitFigcaption($ctx, 'caption'));
        self::assertSame(VisitResult::continue(), $visitor->visitFigureEnd($ctx, ''));

        $result = $visitor->visitText($ctx, 'sample text');
        self::assertSame(VisitResult::continue(), $result);
        self::assertCount(1, $visitor->calls);
        self::assertSame('visitText', $visitor->calls[0][0]);
        self::assertSame('sample text', $visitor->calls[0][1]);
    }

    public function testCustomVisitorForLinkTracking(): void
    {
        $visitor = new class () extends AbstractVisitor {
            /** @var array<int, array{href: string, text: string, title: string|null}> */
            public array $links = [];

            public function visitLink(NodeContext $context, string $href, string $text, ?string $title): array
            {
                $this->links[] = [
                    'href' => $href,
                    'text' => $text,
                    'title' => $title,
                ];
                return VisitResult::continue();
            }
        };

        $ctx = new NodeContext('element', 'a', ['href' => 'https://example.com'], 1, 0, 'p', true);
        $visitor->visitLink($ctx, 'https://example.com', 'Example', 'Example Link');

        self::assertCount(1, $visitor->links);
        self::assertSame('https://example.com', $visitor->links[0]['href']);
        self::assertSame('Example', $visitor->links[0]['text']);
        self::assertSame('Example Link', $visitor->links[0]['title']);
    }

    public function testCustomVisitorForImageTracking(): void
    {
        $visitor = new class () extends AbstractVisitor {
            /** @var array<int, array{src: string, alt: string, title: string|null}> */
            public array $images = [];

            public function visitImage(NodeContext $context, string $src, string $alt, ?string $title): array
            {
                $this->images[] = [
                    'src' => $src,
                    'alt' => $alt,
                    'title' => $title,
                ];
                return VisitResult::continue();
            }
        };

        $ctx = new NodeContext('element', 'img', ['src' => 'test.png', 'alt' => 'test image'], 1, 0, 'p', true);
        $visitor->visitImage($ctx, 'test.png', 'test image', null);

        self::assertCount(1, $visitor->images);
        self::assertSame('test.png', $visitor->images[0]['src']);
        self::assertSame('test image', $visitor->images[0]['alt']);
        self::assertNull($visitor->images[0]['title']);
    }

    public function testCustomVisitorSkippingElements(): void
    {
        $visitor = new class () extends AbstractVisitor {
            public function visitImage(NodeContext $context, string $src, string $alt, ?string $title): array
            {
                return VisitResult::skip();
            }
        };

        $ctx = new NodeContext('element', 'img', [], 1, 0, 'p', true);
        $result = $visitor->visitImage($ctx, 'test.png', 'test', null);

        self::assertSame('skip', $result['type']);
    }

    public function testCustomVisitorPreservingHtml(): void
    {
        $visitor = new class () extends AbstractVisitor {
            public function visitCustomElement(NodeContext $context, string $tagName, string $html): array
            {
                if ($tagName === 'svg') {
                    return VisitResult::preserveHtml();
                }
                return VisitResult::continue();
            }
        };

        $ctx = new NodeContext('element', 'svg', [], 1, 0, 'p', false);
        $result = $visitor->visitCustomElement($ctx, 'svg', '<svg></svg>');

        self::assertSame('preserve_html', $result['type']);
    }

    public function testCustomVisitorWithCustomOutput(): void
    {
        $visitor = new class () extends AbstractVisitor {
            public function visitLink(NodeContext $context, string $href, string $text, ?string $title): array
            {
                return VisitResult::custom("[{$text}]({$href})");
            }
        };

        $ctx = new NodeContext('element', 'a', ['href' => 'https://example.com'], 1, 0, 'p', true);
        $result = $visitor->visitLink($ctx, 'https://example.com', 'Example', null);

        self::assertSame('custom', $result['type']);
        self::assertSame('[Example](https://example.com)', $result['output'] ?? '');
    }

    public function testVisitorForHeadingAnalytics(): void
    {
        $visitor = new class () extends AbstractVisitor {
            /** @var array<int, array{level: int, text: string, id: string|null}> */
            public array $headings = [];

            public function visitHeading(NodeContext $context, int $level, string $text, ?string $id): array
            {
                $this->headings[] = [
                    'level' => $level,
                    'text' => $text,
                    'id' => $id,
                ];
                return VisitResult::continue();
            }
        };

        $ctx = new NodeContext('element', 'h1', ['id' => 'intro'], 1, 0, 'body', false);
        $visitor->visitHeading($ctx, 1, 'Introduction', 'intro');

        $ctx2 = new NodeContext('element', 'h2', [], 2, 0, 'body', false);
        $visitor->visitHeading($ctx2, 2, 'Getting Started', null);

        self::assertCount(2, $visitor->headings);
        self::assertSame(1, $visitor->headings[0]['level']);
        self::assertSame('Introduction', $visitor->headings[0]['text']);
        self::assertSame('intro', $visitor->headings[0]['id']);
        self::assertSame(2, $visitor->headings[1]['level']);
        self::assertSame('Getting Started', $visitor->headings[1]['text']);
        self::assertNull($visitor->headings[1]['id']);
    }

    public function testVisitorForCodeBlockLanguageDetection(): void
    {
        $visitor = new class () extends AbstractVisitor {
            /** @var array<int, array{language: string|null, length: int}> */
            public array $codeBlocks = [];

            public function visitCodeBlock(NodeContext $context, ?string $lang, string $code): array
            {
                $this->codeBlocks[] = [
                    'language' => $lang,
                    'length' => \strlen($code),
                ];
                return VisitResult::continue();
            }
        };

        $ctx = new NodeContext('element', 'pre', ['class' => 'language-php'], 1, 0, 'body', false);
        $visitor->visitCodeBlock($ctx, 'php', 'echo "Hello";');

        $ctx2 = new NodeContext('element', 'pre', [], 1, 1, 'body', false);
        $visitor->visitCodeBlock($ctx2, null, 'plain text code');

        self::assertCount(2, $visitor->codeBlocks);
        self::assertSame('php', $visitor->codeBlocks[0]['language']);
        self::assertSame(13, $visitor->codeBlocks[0]['length']);
        self::assertNull($visitor->codeBlocks[1]['language']);
        self::assertSame(15, $visitor->codeBlocks[1]['length']);
    }

    public function testVisitorForTableStructureAnalysis(): void
    {
        $visitor = new class () extends AbstractVisitor {
            public int $tableCount = 0;
            public int $rowCount = 0;

            public function visitTableStart(NodeContext $context): array
            {
                $this->tableCount++;
                return VisitResult::continue();
            }

            public function visitTableRow(NodeContext $context, array $cells, bool $isHeader): array
            {
                $this->rowCount++;
                return VisitResult::continue();
            }
        };

        $ctx = new NodeContext('element', 'table', [], 1, 0, 'body', false);
        $visitor->visitTableStart($ctx);

        $ctx_row = new NodeContext('element', 'tr', [], 2, 0, 'table', false);
        $visitor->visitTableRow($ctx_row, ['Header 1', 'Header 2'], true);
        $visitor->visitTableRow($ctx_row, ['Cell 1', 'Cell 2'], false);

        self::assertSame(1, $visitor->tableCount);
        self::assertSame(2, $visitor->rowCount);
    }

    public function testVisitorForListProcessing(): void
    {
        $visitor = new class () extends AbstractVisitor {
            /** @var array<int, array{ordered: bool, items: list<string>}> */
            public array $lists = [];
            /** @var array{ordered?: bool, items?: list<string>} */
            public array $currentList = [];

            public function visitListStart(NodeContext $context, bool $ordered): array
            {
                $this->currentList = [
                    'ordered' => $ordered,
                    'items' => [],
                ];
                return VisitResult::continue();
            }

            public function visitListItem(NodeContext $context, bool $ordered, string $marker, string $text): array
            {
                if (!isset($this->currentList['items'])) {
                    $this->currentList['items'] = [];
                }
                $this->currentList['items'][] = $text;
                return VisitResult::continue();
            }

            public function visitListEnd(NodeContext $context, bool $ordered, string $output): array
            {
                if (!empty($this->currentList) && isset($this->currentList['ordered'], $this->currentList['items'])) {
                    /** @var array{ordered: bool, items: list<string>} $current */
                    $current = $this->currentList;
                    $this->lists[] = $current;
                }
                return VisitResult::continue();
            }
        };

        $ctx_list = new NodeContext('element', 'ul', [], 1, 0, 'body', false);
        $visitor->visitListStart($ctx_list, false);

        $ctx_item = new NodeContext('element', 'li', [], 2, 0, 'ul', false);
        $visitor->visitListItem($ctx_item, false, '-', 'First item');
        $visitor->visitListItem($ctx_item, false, '-', 'Second item');

        $visitor->visitListEnd($ctx_list, false, '- First item\n- Second item');

        self::assertCount(1, $visitor->lists);
        $firstList = $visitor->lists[0] ?? [];
        self::assertFalse($firstList['ordered'] ?? false);
        $items = $firstList['items'] ?? [];
        self::assertCount(2, $items);
        self::assertSame('First item', $items[0] ?? null);
        self::assertSame('Second item', $items[1] ?? null);
    }

    /**
     * Test that PHP visitor callbacks are actually invoked during conversion.
     * This verifies the Rust FFI bridge properly calls PHP methods.
     *
     * @group visitor-integration
     */
    public function testPhpVisitorCallbacksAreInvokedDuringConversion(): void
    {
        $this->markTestIncomplete('PHP visitor integration not yet fully implemented in ext-php-rs bindings');
        $callbackTracker = [];
        $visitor = new class ($callbackTracker) extends AbstractVisitor {
            /**
             * @param array<int, array<int|string, mixed>> $tracker
             * @phpstan-ignore-next-line property.unused
             */
            public function __construct(private array &$tracker)
            {
            }

            public function visitText(NodeContext $context, string $text): array
            {
                $this->tracker[] = ['visitText', $text];
                return VisitResult::continue();
            }

            public function visitLink(NodeContext $context, string $href, string $text, ?string $title): array
            {
                $this->tracker[] = ['visitLink', $href, $text, $title];
                return VisitResult::continue();
            }

            public function visitElementStart(NodeContext $context): array
            {
                $this->tracker[] = ['visitElementStart', $context->tagName];
                return VisitResult::continue();
            }

            public function visitElementEnd(NodeContext $context, string $output): array
            {
                $this->tracker[] = ['visitElementEnd', $context->tagName];
                return VisitResult::continue();
            }
        };

        $html = '<p>Hello <a href="https://example.com">World</a></p>';
        $markdown = \HtmlToMarkdown\HtmlToMarkdown::convertWithVisitor($html, null, $visitor);

        self::assertNotEmpty($callbackTracker, 'Visitor callbacks were not invoked during conversion');
        self::assertGreaterThan(0, \count($callbackTracker), 'At least one visitor callback should be invoked');

        $callbackNames = \array_map(fn ($item) => $item[0], $callbackTracker);
        self::assertContains('visitLink', $callbackNames, 'visitLink callback should be invoked for <a> tag');
        self::assertContains('visitText', $callbackNames, 'visitText callback should be invoked for text nodes');

        self::assertStringContainsString('[World](https://example.com)', $markdown);
    }

    /**
     * Test that custom VisitResult returns from callbacks are honored.
     *
     * @group visitor-integration
     */
    public function testPhpVisitorCustomResultsAreHonored(): void
    {
        $this->markTestIncomplete('PHP visitor integration not yet fully implemented in ext-php-rs bindings');
        $visitor = new class () extends AbstractVisitor {
            public function visitLink(NodeContext $context, string $href, string $text, ?string $title): array
            {
                return VisitResult::custom(">>> LINK: {$text} <<<");
            }
        };

        $html = '<p>Check <a href="https://example.com">this link</a> out</p>';
        $markdown = \HtmlToMarkdown\HtmlToMarkdown::convertWithVisitor($html, null, $visitor);

        self::assertStringContainsString('>>> LINK: this link <<<', $markdown);
    }

    /**
     * Test that visitor can skip elements.
     *
     * @group visitor-integration
     */
    public function testPhpVisitorCanSkipElements(): void
    {
        $this->markTestIncomplete('PHP visitor integration not yet fully implemented in ext-php-rs bindings');
        $visitor = new class () extends AbstractVisitor {
            public function visitImage(NodeContext $context, string $src, string $alt, ?string $title): array
            {
                return VisitResult::skip();
            }
        };

        $html = '<p>Before image <img src="test.png" alt="test"/> after image</p>';
        $markdown = \HtmlToMarkdown\HtmlToMarkdown::convertWithVisitor($html, null, $visitor);

        self::assertStringNotContainsString('![', $markdown);
        self::assertStringContainsString('Before image', $markdown);
        self::assertStringContainsString('after image', $markdown);
    }

    /**
     * Test that visitor can preserve HTML for specific elements.
     *
     * @group visitor-integration
     */
    public function testPhpVisitorCanPreserveHtml(): void
    {
        $this->markTestIncomplete('PHP visitor integration not yet fully implemented in ext-php-rs bindings');
        $visitor = new class () extends AbstractVisitor {
            public function visitCustomElement(NodeContext $context, string $tagName, string $html): array
            {
                if ($tagName === 'custom-widget') {
                    return VisitResult::preserveHtml();
                }
                return VisitResult::continue();
            }
        };

        $html = '<p>Before <custom-widget>Widget Content</custom-widget> after</p>';
        $markdown = \HtmlToMarkdown\HtmlToMarkdown::convertWithVisitor($html, null, $visitor);

        self::assertStringContainsString('<custom-widget>', $markdown);
        self::assertStringContainsString('</custom-widget>', $markdown);
    }

    /**
     * Test that multiple visitor callbacks are invoked in correct order.
     *
     * @group visitor-integration
     */
    public function testMultiplePhpVisitorCallbacksInvokedInOrder(): void
    {
        $this->markTestIncomplete('PHP visitor integration not yet fully implemented in ext-php-rs bindings');
        $callOrder = [];
        $visitor = new class ($callOrder) extends AbstractVisitor {
            /**
             * @param array<int, string> $order
             * @phpstan-ignore-next-line property.unused
             */
            public function __construct(private array &$order)
            {
            }

            public function visitElementStart(NodeContext $context): array
            {
                $this->order[] = 'start_' . $context->tagName;
                return VisitResult::continue();
            }

            public function visitElementEnd(NodeContext $context, string $output): array
            {
                $this->order[] = 'end_' . $context->tagName;
                return VisitResult::continue();
            }

            public function visitText(NodeContext $context, string $text): array
            {
                $this->order[] = 'text';
                return VisitResult::continue();
            }
        };

        $html = '<div><p>Hello</p></div>';
        \HtmlToMarkdown\HtmlToMarkdown::convertWithVisitor($html, null, $visitor);

        self::assertNotEmpty($callOrder);
        self::assertGreaterThan(0, \count(\array_filter($callOrder, fn ($c) => \str_starts_with($c, 'start_'))));
    }
}
