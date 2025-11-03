<?php

declare(strict_types=1);

namespace HtmlToMarkdown\Config;

use HtmlToMarkdown\Enum\CodeBlockStyle;
use HtmlToMarkdown\Enum\HeadingStyle;
use HtmlToMarkdown\Enum\HighlightStyle;
use HtmlToMarkdown\Enum\ListIndentType;
use HtmlToMarkdown\Enum\NewlineStyle;
use HtmlToMarkdown\Enum\WhitespaceMode;
use HtmlToMarkdown\Exception\InvalidOption;
use HtmlToMarkdown\Internal\TypeAssertions;

final readonly class ConversionOptions
{
    public PreprocessingOptions $preprocessing;

    /**
     * @param list<string> $keepInlineImagesIn
     * @param list<string> $stripTags
     * @param list<string> $preserveTags
     */
    public function __construct(
        public HeadingStyle $headingStyle = HeadingStyle::ATX,
        public ListIndentType $listIndentType = ListIndentType::SPACES,
        public int $listIndentWidth = 2,
        public string $bullets = '-',
        public string $strongEmSymbol = '*',
        public bool $escapeAsterisks = false,
        public bool $escapeUnderscores = false,
        public bool $escapeMisc = false,
        public bool $escapeAscii = false,
        public string $codeLanguage = '',
        public bool $autolinks = true,
        public bool $defaultTitle = false,
        public bool $brInTables = false,
        public bool $hocrSpatialTables = true,
        public HighlightStyle $highlightStyle = HighlightStyle::DOUBLE_EQUAL,
        public bool $extractMetadata = true,
        public WhitespaceMode $whitespaceMode = WhitespaceMode::NORMALIZED,
        public bool $stripNewlines = false,
        public bool $wrap = false,
        public int $wrapWidth = 80,
        public bool $convertAsInline = false,
        public string $subSymbol = '',
        public string $supSymbol = '',
        public NewlineStyle $newlineStyle = NewlineStyle::SPACES,
        public CodeBlockStyle $codeBlockStyle = CodeBlockStyle::INDENTED,
        public array $keepInlineImagesIn = [],
        public string $encoding = 'utf-8',
        public bool $debug = false,
        public array $stripTags = [],
        public array $preserveTags = [],
        ?PreprocessingOptions $preprocessing = null,
    ) {
        if ($this->listIndentWidth < 0) {
            throw InvalidOption::because('list_indent_width', 'must be non-negative');
        }

        $this->assertSingleCharacter($strongEmSymbol, 'strong_em_symbol');
        $this->preprocessing = $preprocessing ?? new PreprocessingOptions();
    }

    /**
     * @param array<string, mixed> $input
     */
    public static function fromArray(array $input): self
    {
        $defaults = new self();

        return new self(
            headingStyle: array_key_exists('heading_style', $input)
                ? HeadingStyle::fromString(TypeAssertions::string($input['heading_style'], 'heading_style'))
                : $defaults->headingStyle,
            listIndentType: array_key_exists('list_indent_type', $input)
                ? ListIndentType::fromString(TypeAssertions::string($input['list_indent_type'], 'list_indent_type'))
                : $defaults->listIndentType,
            listIndentWidth: array_key_exists('list_indent_width', $input)
                ? TypeAssertions::positiveInt($input['list_indent_width'], 'list_indent_width')
                : $defaults->listIndentWidth,
            bullets: array_key_exists('bullets', $input)
                ? TypeAssertions::string($input['bullets'], 'bullets')
                : $defaults->bullets,
            strongEmSymbol: array_key_exists('strong_em_symbol', $input)
                ? self::normalizeStrongSymbol(TypeAssertions::string($input['strong_em_symbol'], 'strong_em_symbol'))
                : $defaults->strongEmSymbol,
            escapeAsterisks: array_key_exists('escape_asterisks', $input)
                ? TypeAssertions::bool($input['escape_asterisks'], 'escape_asterisks')
                : $defaults->escapeAsterisks,
            escapeUnderscores: array_key_exists('escape_underscores', $input)
                ? TypeAssertions::bool($input['escape_underscores'], 'escape_underscores')
                : $defaults->escapeUnderscores,
            escapeMisc: array_key_exists('escape_misc', $input)
                ? TypeAssertions::bool($input['escape_misc'], 'escape_misc')
                : $defaults->escapeMisc,
            escapeAscii: array_key_exists('escape_ascii', $input)
                ? TypeAssertions::bool($input['escape_ascii'], 'escape_ascii')
                : $defaults->escapeAscii,
            codeLanguage: array_key_exists('code_language', $input)
                ? TypeAssertions::string($input['code_language'], 'code_language')
                : $defaults->codeLanguage,
            autolinks: array_key_exists('autolinks', $input)
                ? TypeAssertions::bool($input['autolinks'], 'autolinks')
                : $defaults->autolinks,
            defaultTitle: array_key_exists('default_title', $input)
                ? TypeAssertions::bool($input['default_title'], 'default_title')
                : $defaults->defaultTitle,
            brInTables: array_key_exists('br_in_tables', $input)
                ? TypeAssertions::bool($input['br_in_tables'], 'br_in_tables')
                : $defaults->brInTables,
            hocrSpatialTables: array_key_exists('hocr_spatial_tables', $input)
                ? TypeAssertions::bool($input['hocr_spatial_tables'], 'hocr_spatial_tables')
                : $defaults->hocrSpatialTables,
            highlightStyle: array_key_exists('highlight_style', $input)
                ? HighlightStyle::fromString(TypeAssertions::string($input['highlight_style'], 'highlight_style'))
                : $defaults->highlightStyle,
            extractMetadata: array_key_exists('extract_metadata', $input)
                ? TypeAssertions::bool($input['extract_metadata'], 'extract_metadata')
                : $defaults->extractMetadata,
            whitespaceMode: array_key_exists('whitespace_mode', $input)
                ? WhitespaceMode::fromString(TypeAssertions::string($input['whitespace_mode'], 'whitespace_mode'))
                : $defaults->whitespaceMode,
            stripNewlines: array_key_exists('strip_newlines', $input)
                ? TypeAssertions::bool($input['strip_newlines'], 'strip_newlines')
                : $defaults->stripNewlines,
            wrap: array_key_exists('wrap', $input)
                ? TypeAssertions::bool($input['wrap'], 'wrap')
                : $defaults->wrap,
            wrapWidth: array_key_exists('wrap_width', $input)
                ? TypeAssertions::positiveInt($input['wrap_width'], 'wrap_width')
                : $defaults->wrapWidth,
            convertAsInline: array_key_exists('convert_as_inline', $input)
                ? TypeAssertions::bool($input['convert_as_inline'], 'convert_as_inline')
                : $defaults->convertAsInline,
            subSymbol: array_key_exists('sub_symbol', $input)
                ? TypeAssertions::string($input['sub_symbol'], 'sub_symbol')
                : $defaults->subSymbol,
            supSymbol: array_key_exists('sup_symbol', $input)
                ? TypeAssertions::string($input['sup_symbol'], 'sup_symbol')
                : $defaults->supSymbol,
            newlineStyle: array_key_exists('newline_style', $input)
                ? NewlineStyle::fromString(TypeAssertions::string($input['newline_style'], 'newline_style'))
                : $defaults->newlineStyle,
            codeBlockStyle: array_key_exists('code_block_style', $input)
                ? CodeBlockStyle::fromString(TypeAssertions::string($input['code_block_style'], 'code_block_style'))
                : $defaults->codeBlockStyle,
            keepInlineImagesIn: array_key_exists('keep_inline_images_in', $input)
                ? TypeAssertions::stringList($input['keep_inline_images_in'], 'keep_inline_images_in')
                : $defaults->keepInlineImagesIn,
            encoding: array_key_exists('encoding', $input)
                ? TypeAssertions::string($input['encoding'], 'encoding')
                : $defaults->encoding,
            debug: array_key_exists('debug', $input)
                ? TypeAssertions::bool($input['debug'], 'debug')
                : $defaults->debug,
            stripTags: array_key_exists('strip_tags', $input)
                ? TypeAssertions::stringList($input['strip_tags'], 'strip_tags')
                : $defaults->stripTags,
            preserveTags: array_key_exists('preserve_tags', $input)
                ? TypeAssertions::stringList($input['preserve_tags'], 'preserve_tags')
                : $defaults->preserveTags,
            preprocessing: array_key_exists('preprocessing', $input)
                ? PreprocessingOptions::fromArray(self::normalizeArray($input['preprocessing'], 'preprocessing'))
                : $defaults->preprocessing,
        );
    }

    /**
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        return [
            'heading_style' => $this->headingStyle->value,
            'list_indent_type' => $this->listIndentType->value,
            'list_indent_width' => $this->listIndentWidth,
            'bullets' => $this->bullets,
            'strong_em_symbol' => $this->strongEmSymbol,
            'escape_asterisks' => $this->escapeAsterisks,
            'escape_underscores' => $this->escapeUnderscores,
            'escape_misc' => $this->escapeMisc,
            'escape_ascii' => $this->escapeAscii,
            'code_language' => $this->codeLanguage,
            'autolinks' => $this->autolinks,
            'default_title' => $this->defaultTitle,
            'br_in_tables' => $this->brInTables,
            'hocr_spatial_tables' => $this->hocrSpatialTables,
            'highlight_style' => $this->highlightStyle->value,
            'extract_metadata' => $this->extractMetadata,
            'whitespace_mode' => $this->whitespaceMode->value,
            'strip_newlines' => $this->stripNewlines,
            'wrap' => $this->wrap,
            'wrap_width' => $this->wrapWidth,
            'convert_as_inline' => $this->convertAsInline,
            'sub_symbol' => $this->subSymbol,
            'sup_symbol' => $this->supSymbol,
            'newline_style' => $this->newlineStyle->value,
            'code_block_style' => $this->codeBlockStyle->value,
            'keep_inline_images_in' => array_values($this->keepInlineImagesIn),
            'preprocessing' => $this->preprocessing->toArray(),
            'encoding' => $this->encoding,
            'debug' => $this->debug,
            'strip_tags' => array_values($this->stripTags),
            'preserve_tags' => array_values($this->preserveTags),
        ];
    }

    /**
     * @return array<string, mixed>
     */
    private static function normalizeArray(mixed $value, string $key): array
    {
        if (!is_array($value)) {
            throw InvalidOption::because($key, sprintf('expected array, got %s', get_debug_type($value)));
        }

        return $value;
    }

    private static function normalizeStrongSymbol(string $value): string
    {
        $length = mb_strlen($value, 'UTF-8');
        if ($length !== 1) {
            throw InvalidOption::because('strong_em_symbol', 'must be a single character');
        }

        return $value;
    }

    private function assertSingleCharacter(string $value, string $option): void
    {
        $length = mb_strlen($value, 'UTF-8');
        if ($length !== 1) {
            throw InvalidOption::because($option, 'must be a single character');
        }
    }
}
