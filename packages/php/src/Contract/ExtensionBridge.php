<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Contract;

interface ExtensionBridge
{
    /**
     * @param array<string, mixed>|null $options
     */
    public function convert(string $html, ?array $options = null): string;

    /**
     * @param array<string, mixed>|null $options
     * @param array<string, mixed>|null $config
     *
     * @return array<string, mixed>
     */
    public function convertWithInlineImages(
        string $html,
        ?array $options = null,
        ?array $config = null,
    ): array;
}
