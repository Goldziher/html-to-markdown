<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Config;

use HtmlToMarkdown\Enum\PreprocessingPreset;
use HtmlToMarkdown\Exception\InvalidOption;
use HtmlToMarkdown\Internal\TypeAssertions;

final readonly class PreprocessingOptions
{
    public function __construct(
        public bool $enabled = true,
        public PreprocessingPreset $preset = PreprocessingPreset::STANDARD,
        public bool $removeNavigation = true,
        public bool $removeForms = true,
    ) {
        if (($removeNavigation || $removeForms) && !$enabled) {
            throw InvalidOption::because(
                'preprocessing',
                'remove_navigation/remove_forms require preprocessing to be enabled',
            );
        }
    }

    /**
     * @param array<string, mixed> $input
     */
    public static function fromArray(array $input): self
    {
        $defaults = new self();

        return new self(
            enabled: \array_key_exists('enabled', $input)
                ? TypeAssertions::bool($input['enabled'], 'preprocessing.enabled')
                : $defaults->enabled,
            preset: \array_key_exists('preset', $input)
                ? PreprocessingPreset::fromString(TypeAssertions::string($input['preset'], 'preprocessing.preset'))
                : $defaults->preset,
            removeNavigation: \array_key_exists('remove_navigation', $input)
                ? TypeAssertions::bool($input['remove_navigation'], 'preprocessing.remove_navigation')
                : $defaults->removeNavigation,
            removeForms: \array_key_exists('remove_forms', $input)
                ? TypeAssertions::bool($input['remove_forms'], 'preprocessing.remove_forms')
                : $defaults->removeForms,
        );
    }

    public function withPreset(PreprocessingPreset $preset): self
    {
        return new self(
            enabled: $this->enabled,
            preset: $preset,
            removeNavigation: $this->removeNavigation,
            removeForms: $this->removeForms,
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        return [
            'enabled' => $this->enabled,
            'preset' => $this->preset->value,
            'remove_navigation' => $this->removeNavigation,
            'remove_forms' => $this->removeForms,
        ];
    }

    public function isDefault(): bool
    {
        static $defaults = null;
        $defaults ??= new self();

        return $this->enabled === $defaults->enabled
            && $this->preset === $defaults->preset
            && $this->removeNavigation === $defaults->removeNavigation
            && $this->removeForms === $defaults->removeForms;
    }

}
