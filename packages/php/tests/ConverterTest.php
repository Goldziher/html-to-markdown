<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Tests;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Service\Converter;
use HtmlToMarkdown\Value\InlineImageExtraction;
use HtmlToMarkdown\Value\InlineImageFormat;
use HtmlToMarkdown\Value\InlineImageWarning;
use PHPUnit\Framework\TestCase;

final class ConverterTest extends TestCase
{
    private Converter $converter;

    protected function setUp(): void
    {
        parent::setUp();
        $this->converter = Converter::create();
    }

    public function testConvertReturnsMarkdown(): void
    {
        $markdown = $this->converter->convert('<h1>Hello</h1>');

        self::assertSame("# Hello\n", $markdown);
    }

    public function testConvertWithOptionsChangesOutput(): void
    {
        $options = new ConversionOptions(wrap: true, wrapWidth: 10);
        $markdown = $this->converter->convert('<p>Lorem ipsum dolor sit amet</p>', $options);

        self::assertSame("Lorem\nipsum\ndolor sit\namet\n\n", $markdown);
    }

    public function testConvertWithInlineImagesReturnsExtraction(): void
    {
        $html = '<p><img src="data:image/png;base64,Zm9v" alt="test"></p>';

        $extraction = $this->converter->convertWithInlineImages($html, config: new InlineImageConfig());

        self::assertInstanceOf(InlineImageExtraction::class, $extraction);
        self::assertSame("![test](data:image/png;base64,Zm9v)\n", $extraction->markdown);
        self::assertCount(1, $extraction->inlineImages);

        $image = $extraction->inlineImages[0];
        self::assertInstanceOf(InlineImageFormat::class, $image->format);
        self::assertTrue($image->format->equals(InlineImageFormat::fromString('png')));
        self::assertSame('foo', $image->data);
        self::assertSame('embedded_image_1.png', $image->filename);

        self::assertSame([], $extraction->warnings);
    }

    public function testWarningsAreCollected(): void
    {
        $html = '<p><img src="data:image/png;base64,Z" alt="bad"></p>';

        $extraction = $this->converter->convertWithInlineImages($html);

        self::assertInstanceOf(InlineImageExtraction::class, $extraction);
        self::assertCount(0, $extraction->inlineImages);
        self::assertNotEmpty($extraction->warnings);
        self::assertInstanceOf(InlineImageWarning::class, $extraction->warnings[0]);
    }
}
