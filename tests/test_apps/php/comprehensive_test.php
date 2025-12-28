<?php

declare(strict_types=1);

use PHPUnit\Framework\TestCase;

final class ComprehensiveTest extends TestCase
{
    private function loadFixtures(string $filename): array
    {
        $path = __DIR__ . '/../fixtures/' . $filename;
        return json_decode(file_get_contents($path), true);
    }

    /**
     * @dataProvider basicHtmlProvider
     */
    public function testBasicHtmlConversion(string $name, string $html, string $expected, array $options): void
    {
        $result = html_to_markdown_convert($html, $options);
        $this->assertSame(trim($expected), trim($result), $name);
    }

    public function basicHtmlProvider(): array
    {
        $fixtures = $this->loadFixtures('basic-html.json');
        $cases = [];
        foreach ($fixtures as $fixture) {
            $cases[$fixture['name']] = [
                $fixture['name'],
                $fixture['html'],
                $fixture['expectedMarkdown'],
                $fixture['options'] ?? []
            ];
        }
        return $cases;
    }
}
