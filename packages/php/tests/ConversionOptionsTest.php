<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Tests;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\PreprocessingOptions;
use HtmlToMarkdown\Enum\CodeBlockStyle;
use HtmlToMarkdown\Enum\HeadingStyle;
use HtmlToMarkdown\Enum\ListIndentType;
use HtmlToMarkdown\Enum\PreprocessingPreset;
use HtmlToMarkdown\Exception\InvalidOption;
use PHPUnit\Framework\TestCase;

final class ConversionOptionsTest extends TestCase
{
    public function testFromArrayParsesEnums(): void
    {
        $options = ConversionOptions::fromArray([
            'heading_style' => 'underlined',
            'list_indent_type' => 'tabs',
            'code_block_style' => 'backticks',
            'preprocessing' => [
                'enabled' => true,
                'preset' => 'minimal',
                'remove_navigation' => false,
                'remove_forms' => true,
            ],
        ]);

        self::assertSame(HeadingStyle::UNDERLINED, $options->headingStyle);
        self::assertSame(ListIndentType::TABS, $options->listIndentType);
        self::assertSame(CodeBlockStyle::BACKTICKS, $options->codeBlockStyle);
        self::assertEquals(
            new PreprocessingOptions(
                enabled: true,
                preset: PreprocessingPreset::MINIMAL,
                removeNavigation: false,
                removeForms: true,
            ),
            $options->preprocessing,
        );
    }

    public function testInvalidStrongEmSymbolThrows(): void
    {
        $this->expectException(InvalidOption::class);
        ConversionOptions::fromArray(['strong_em_symbol' => 'too-long']);
    }
}
