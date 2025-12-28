<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class SmokeTest extends TestCase
{
    public function testPackageLoads(): void
    {
        $this->assertTrue(extension_loaded('html_to_markdown'));
    }

    public function testBasicConversion(): void
    {
        $html = '<p>Hello World</p>';
        $result = html_to_markdown_convert($html);
        $this->assertStringContainsString('Hello World', $result);
    }

    public function testWithOptions(): void
    {
        $html = '<h1>Title</h1>';
        $result = html_to_markdown_convert($html);
        $this->assertStringStartsWith('#', $result);
    }

    public function testEmptyInput(): void
    {
        $result = html_to_markdown_convert('');
        $this->assertSame('', $result);
    }
}
