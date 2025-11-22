<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Tests;

use function HtmlToMarkdown\convert;
use function HtmlToMarkdown\convert_with_inline_images;

use HtmlToMarkdown\HtmlToMarkdown;
use HtmlToMarkdown\Value\InlineImageExtraction;

final class FunctionsTest extends TestCase
{
    public function testConvertHelperAcceptsArray(): void
    {
        $markdown = convert('<p>Hello</p>', ['heading_style' => 'atx']);

        self::assertSame("Hello\n", $markdown);
    }

    public function testConvertWithInlineImagesHelperAcceptsArray(): void
    {
        $result = convert_with_inline_images('<img src="data:image/png;base64,Zm9v" alt="x">', null, [
            'infer_dimensions' => false,
        ]);

        self::assertInstanceOf(InlineImageExtraction::class, $result);
        self::assertSame("![x](data:image/png;base64,Zm9v)\n", $result->markdown);
    }

    public function testFacadeAcceptsArray(): void
    {
        $markdown = HtmlToMarkdown::convert('<p>Hello</p>', ['heading_style' => 'atx']);

        self::assertSame("Hello\n", $markdown);
    }
}
