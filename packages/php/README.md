# html-to-markdown PHP package

[![Crates.io](https://img.shields.io/crates/v/html-to-markdown-rs.svg?logo=rust&label=crates.io)](https://crates.io/crates/html-to-markdown-rs)
[![npm (node)](https://img.shields.io/npm/v/html-to-markdown-node.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-node)
[![npm (wasm)](https://img.shields.io/npm/v/html-to-markdown-wasm.svg?logo=npm)](https://www.npmjs.com/package/html-to-markdown-wasm)
[![PyPI](https://img.shields.io/pypi/v/html-to-markdown.svg?logo=pypi)](https://pypi.org/project/html-to-markdown/)
[![Packagist](https://img.shields.io/packagist/v/goldziher/html-to-markdown.svg)](https://packagist.org/packages/goldziher/html-to-markdown)
[![RubyGems](https://badge.fury.io/rb/html-to-markdown.svg)](https://rubygems.org/gems/html-to-markdown)
[![Hex.pm](https://img.shields.io/hexpm/v/html_to_markdown.svg)](https://hex.pm/packages/html_to_markdown)
[![NuGet](https://img.shields.io/nuget/v/Goldziher.HtmlToMarkdown.svg)](https://www.nuget.org/packages/Goldziher.HtmlToMarkdown/)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.goldziher/html-to-markdown.svg)](https://central.sonatype.com/artifact/io.github.goldziher/html-to-markdown)
[![Go Reference](https://pkg.go.dev/badge/github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown.svg)](https://pkg.go.dev/github.com/Goldziher/html-to-markdown/packages/go/v2/htmltomarkdown)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Goldziher/html-to-markdown/blob/main/LICENSE)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

Typed PHP bindings for the `html_to_markdown` native extension generated from
the Rust html-to-markdown engine. The API mirrors the behaviour exposed by the
Rust, Python, Ruby, Node.js, and WASM distributions while embracing modern PHP
features (readonly value objects, enums, typed exceptions).

## Requirements

- PHP 8.2 or newer
- `html_to_markdown` native extension (install via PIE `goldziher/html-to-markdown` or download the GitHub release artifacts)
- Composer for dependency management

## Platform Support

The native extension builds and runs on **Linux and macOS** (x86_64 and ARM64).

**Windows is not currently supported** due to limitations in the ext-php-rs library that prevent proper symbol resolution during the build process. Specifically, the Windows PHP SDK does not export certain Zend engine symbols (e.g., `zend_ce_exception`, `zend_ce_traversable`) in a way that the Rust linker can resolve them. This is a known limitation of ext-php-rs 0.15.x on Windows.

If you require Windows support, consider using one of the alternative distributions:
- **Node.js** (`html-to-markdown-node`)
- **Python** (`html-to-markdown`)
- **WebAssembly** (`html-to-markdown-wasm`)

## Installation

```bash
pie install goldziher/html-to-markdown        # Native extension via PIE
composer require goldziher/html-to-markdown   # Typed PHP API wrappers
```

If the extension is not enabled the post-install script prints the steps
required to install or activate it. Once enabled, ensure `extension=html_to_markdown`
is present in `php.ini`.

## Quick start

```php
use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Service\Converter;
use function HtmlToMarkdown\convert;
use function HtmlToMarkdown\convert_with_inline_images;

// Object-oriented usage
$converter = Converter::create();
$markdown = $converter->convert('<h1>Hello</h1>');

$extraction = $converter->convertWithInlineImages(
    '<img src="data:image/png;base64,Zm9v" alt="demo">',
    new ConversionOptions(),
    new InlineImageConfig(inferDimensions: true),
);

// Procedural helpers delegate to the converter
$markdown = convert('<p>Lorem ipsum</p>');
$extraction = convert_with_inline_images('<img src="data:image/png;base64,Zm9v" alt="demo">');
```

## Performance (Apple M4)

Captured via `task bench:harness` so they match the data in the root README:

| Document               | Size   | ops/sec (PHP) |
| ---------------------- | ------ | ------------- |
| Lists (Timeline)       | 129 KB | 533           |
| Tables (Countries)     | 360 KB | 118           |
| Medium (Python)        | 657 KB | 59            |
| Large (Rust)           | 567 KB | 65            |
| Small (Intro)          | 463 KB | 83            |
| hOCR German PDF        | 44 KB  | 1,007         |
| hOCR Invoice           | 4 KB   | 8,781         |
| hOCR Embedded Tables   | 37 KB  | 1,194         |

> Throughput sits in the tens of MB/s range once the extension is loaded; startup time is dominated by compiling the Rust library.

## Configuration

- `ConversionOptions` is an immutable value object covering all options exposed
  by the Rust converter (heading style, wrapping, escaping, table behaviour,
  etc.). Enumerated values use backed enums such as `HeadingStyle::ATX` or
  `CodeBlockStyle::TILDES`.
- `InlineImageConfig` controls inline image extraction (max decoded size,
  filename prefix, SVG capture, dimension inference).
- `PreprocessingOptions` configures sanitisation behaviour prior to conversion.

Each object provides a `toArray()` method and is serialisation-friendly.

## Inline image extraction

`Converter::convertWithInlineImages()` returns an `InlineImageExtraction`
instance containing:

- `markdown`: rendered Markdown string
- `inlineImages`: list of `InlineImage` objects exposing data, format,
  metadata, dimensions, source (`InlineImageSource` enum), and attributes
- `warnings`: list of `InlineImageWarning` objects describing recoverable
  issues (invalid payloads, skipped assets, etc.)

Use `InlineImageFormat` to inspect the image type and `InlineImageDimensions`
for width/height when available.

## Metadata extraction

Extract document structure, embedded links, images, and Open Graph metadata alongside Markdown conversion. All metadata is returned as immutable readonly value objects with full PHPStan type safety.

### Quick Start

```php
<?php

use HtmlToMarkdown\Config\ConversionOptions;
use function HtmlToMarkdown\convert_with_metadata;

$html = <<<'HTML'
<html>
  <head>
    <title>Example Article</title>
    <meta name="description" content="Demo page with metadata">
    <link rel="canonical" href="https://example.com/page">
  </head>
  <body>
    <h1 id="welcome">Welcome</h1>
    <a href="https://example.com" rel="nofollow external">Example link</a>
    <img src="https://example.com/image.jpg" alt="Hero" width="640" height="480">
  </body>
</html>
HTML;

// Procedural API
$result = convert_with_metadata(
    $html,
    new ConversionOptions(headingStyle: 'Atx'),
    ['extract_headers' => true, 'extract_links' => true, 'extract_images' => true],
);

echo $result['markdown'];                                   // # Welcome...
echo $result['metadata']->document->title;                  // "Example Article"
echo implode(', ', $result['metadata']->links[0]->rel);     // "nofollow, external"
echo $result['metadata']->images[0]->dimensions[0];         // 640 (width)

// Object-oriented API
$converter = \HtmlToMarkdown\Service\Converter::create();
$result = $converter->convertWithMetadata(
    $html,
    new ConversionOptions(headingStyle: 'Atx'),
    ['extract_links' => true, 'extract_images' => true],
);
```

### Metadata Configuration

Toggle extraction sections via the `$metadataConfig` array (associative):

```php
<?php

use HtmlToMarkdown\Service\Converter;
use HtmlToMarkdown\Config\ConversionOptions;

$converter = Converter::create();

$result = $converter->convertWithMetadata(
    $html,
    new ConversionOptions(),
    [
        'extract_headers'       => true,      // h1-h6 with depth/offset
        'extract_links'         => true,      // <a> tags with rel + attributes
        'extract_images'        => true,      // <img> with dimensions
        'extract_structured_data' => true,   // JSON-LD & microdata
    ]
);
```

All extraction flags default to `false`; only enabled sections are populated in metadata. Set `$metadataConfig = null` or omit flags to get empty collections.

### Metadata Structure (ExtendedMetadata Value Object)

The returned `metadata` is an immutable `ExtendedMetadata` containing five sections:

#### 1. Document Metadata

```php
<?php

use HtmlToMarkdown\Service\Converter;

$converter = Converter::create();
$result = $converter->convertWithMetadata(
    <<<'HTML'
    <html>
      <head>
        <title>Blog Post</title>
        <meta name="description" content="A detailed guide">
        <meta name="keywords" content="php, conversion">
        <meta name="author" content="Jane Doe">
        <link rel="canonical" href="https://blog.example.com/post/123">
        <base href="https://assets.example.com/">
        <meta property="og:title" content="Blog: PHP Guide">
        <meta property="og:image" content="https://example.com/og.jpg">
        <meta name="twitter:card" content="summary_large_image">
        <meta name="twitter:creator" content="@janedoe">
      </head>
      <body><h1>Content</h1></body>
    </html>
    HTML,
    null,
    null  // No extraction flags, but document is always available
);

$doc = $result['metadata']->document;

// Basic document fields
echo $doc->title;              // "Blog Post" | null
echo $doc->description;        // "A detailed guide" | null
echo implode(', ', $doc->keywords);  // "php, conversion" | null
echo $doc->author;             // "Jane Doe" | null
echo $doc->canonicalUrl;       // "https://blog.example.com/post/123" | null
echo $doc->baseHref;           // "https://assets.example.com/" | null
echo $doc->language;           // "en" | null (from <html lang="en">)
echo $doc->textDirection;      // "ltr" | "rtl" | null (from <html dir="rtl">)

// Open Graph tags (property="og:*")
$og = $doc->openGraph;  // array<string, string>
echo $og['title'] ?? '';       // "Blog: PHP Guide"
echo $og['image'] ?? '';       // "https://example.com/og.jpg"

// Twitter Card tags
$twitter = $doc->twitterCard;  // array<string, string>
echo $twitter['card'] ?? '';   // "summary_large_image"
echo $twitter['creator'] ?? ''; // "@janedoe"

// All <meta> tags (name & property attributes)
$allMeta = $doc->metaTags;     // array<string, string>
echo $allMeta['description'] ?? '';
echo $allMeta['og:title'] ?? '';
```

#### 2. Header/Heading Metadata

Extract all headings with hierarchy depth and position:

```php
<?php

use HtmlToMarkdown\Service\Converter;

$converter = Converter::create();
$result = $converter->convertWithMetadata(
    <<<'HTML'
    <html>
      <body>
        <h1 id="intro">Introduction</h1>
        <p>Paragraph 1</p>
        <h2 id="section-a">Section A</h2>
        <h3 id="subsection">Subsection</h3>
        <h2 id="section-b">Section B</h2>
      </body>
    </html>
    HTML,
    null,
    ['extract_headers' => true],
);

foreach ($result['metadata']->headers as $header) {
    // Each header is a readonly HeaderMetadata object
    echo str_repeat('  ', $header->depth);  // Indentation based on hierarchy
    echo "{$header->level}: {$header->text}\n";

    // All properties:
    echo $header->level;       // 1-6 (h1-h6)
    echo $header->text;        // "Introduction" | "Section A" | ...
    echo $header->id;          // "intro" | null
    echo $header->depth;       // 0 for h1, 1 for h2 under h1, 2 for h3, etc.
    echo $header->htmlOffset;  // Character offset in original HTML
}

// Output:
// 1: Introduction
//   2: Section A
//     3: Subsection
//   2: Section B
```

Use headers for table of contents generation, document outlining, or validation.

#### 3. Link Metadata

Extract all hyperlinks with relationship types and attributes:

```php
<?php

use HtmlToMarkdown\Service\Converter;

$converter = Converter::create();
$result = $converter->convertWithMetadata(
    <<<'HTML'
    <html>
      <body>
        <a href="https://example.com"
           title="Main Site"
           rel="nofollow external">External Link</a>
        <a href="/internal" rel="prefetch">Internal</a>
        <a href="mailto:support@example.com">Email</a>
        <a href="#section">Fragment</a>
      </body>
    </html>
    HTML,
    null,
    ['extract_links' => true],
);

foreach ($result['metadata']->links as $link) {
    // Each link is a readonly LinkMetadata object

    // Core properties
    echo $link->href;          // "https://example.com" | "/internal" | "mailto:..." | "#section"
    echo $link->text;          // "External Link" | "Internal" | "Email" | "Fragment"
    echo $link->title;         // "Main Site" | null
    echo $link->linkType;      // "external" | "internal" | "email" | "fragment"

    // Relationship (rel attribute as list)
    echo implode(', ', $link->rel);  // "nofollow external" | "prefetch" | [] | []

    // Raw HTML attributes
    echo json_encode($link->attributes);  // {"href": "...", "title": "...", "rel": "..."}
}
```

Useful for:
- Link extraction for SEO analysis
- Finding `rel="nofollow"` or `rel="sponsored"` links
- Email/phone/fragment detection
- Building sitemaps (internal links only)

#### 4. Image Metadata

Extract all images with alt text, source, and inferred dimensions:

```php
<?php

use HtmlToMarkdown\Service\Converter;

$converter = Converter::create();
$result = $converter->convertWithMetadata(
    <<<'HTML'
    <html>
      <body>
        <img src="https://example.com/hero.jpg"
             alt="Hero Image"
             width="1200"
             height="600"
             title="Hero Banner">
        <img src="/local/thumb.png" alt="">
        <picture>
          <source srcset="large.webp" media="(min-width: 768px)">
          <img src="small.png" alt="Responsive">
        </picture>
      </body>
    </html>
    HTML,
    null,
    ['extract_images' => true],
);

foreach ($result['metadata']->images as $image) {
    // Each image is a readonly ImageMetadata object

    // Image properties
    echo $image->src;          // "https://example.com/hero.jpg" | "/local/thumb.png"
    echo $image->alt;          // "Hero Image" | "" | null
    echo $image->title;        // "Hero Banner" | null
    echo $image->imageType;    // "external_url" | "internal_url" | "data_uri"

    // Dimensions as [width, height] tuple (null if not available)
    if ($image->dimensions) {
        [$width, $height] = $image->dimensions;
        echo "$width x $height";  // "1200 x 600"
    }

    // Raw HTML attributes (width, height, loading, decoding, etc.)
    echo json_encode($image->attributes);
    // {"src": "...", "alt": "...", "width": "1200", "height": "600", "title": "..."}
}
```

Useful for:
- Image inventory and accessibility audit
- Responsive image analysis
- SEO: checking alt text coverage
- Building image galleries

#### 5. Structured Data (JSON-LD, Microdata, RDFa)

Extract embedded JSON-LD scripts, microdata, and RDFa:

```php
<?php

use HtmlToMarkdown\Service\Converter;

$converter = Converter::create();
$result = $converter->convertWithMetadata(
    <<<'HTML'
    <html>
      <head>
        <script type="application/ld+json">
        {
          "@context": "https://schema.org",
          "@type": "BlogPosting",
          "headline": "Learning PHP Conversion",
          "author": {"@type": "Person", "name": "Jane Doe"},
          "datePublished": "2024-01-15"
        }
        </script>
      </head>
      <body>
        <article itemscope itemtype="https://schema.org/NewsArticle">
          <h1 itemprop="headline">Article Title</h1>
          <span itemprop="author">John Doe</span>
        </article>
      </body>
    </html>
    HTML,
    null,
    ['extract_structured_data' => true],
);

foreach ($result['metadata']->structuredData as $data) {
    // Each structured data is a readonly StructuredData object

    echo $data->dataType;      // "json_ld" | "microdata" | "rdfa"
    echo $data->rawJson;       // Full JSON string (even for microdata/RDFa)
    echo $data->schemaType;    // "BlogPosting" | "NewsArticle" | null

    // Parse JSON for application logic
    $parsed = json_decode($data->rawJson, associative: true);
    echo $parsed['headline'] ?? '';  // "Learning PHP Conversion"
}
```

Useful for:
- Extracting SEO schema (Article, Product, Recipe, etc.)
- Validating structured data markup
- Building Knowledge Graph content
- Analytics and enrichment

### Real-World Examples

#### Example 1: SEO Audit Tool

```php
<?php

declare(strict_types=1);

use HtmlToMarkdown\Service\Converter;
use HtmlToMarkdown\Config\ConversionOptions;

class SeoAudit
{
    private readonly Converter $converter;

    public function __construct()
    {
        $this->converter = Converter::create();
    }

    /**
     * @return array{
     *     title_missing: bool,
     *     description_missing: bool,
     *     og_missing: array<string>,
     *     headings: array<string>,
     *     images_missing_alt: array<string>,
     *     external_links: array<string>,
     *     issues: array<string>,
     * }
     */
    public function audit(string $html): array
    {
        $result = $this->converter->convertWithMetadata(
            $html,
            new ConversionOptions(),
            [
                'extract_headers' => true,
                'extract_links' => true,
                'extract_images' => true,
            ]
        );

        $doc = $result['metadata']->document;
        $issues = [];

        // Title check
        $titleMissing = !$doc->title || strlen($doc->title) === 0;
        if ($titleMissing) {
            $issues[] = "Missing page title";
        }

        // Meta description check
        $descMissing = !$doc->description || strlen($doc->description) === 0;
        if ($descMissing) {
            $issues[] = "Missing meta description";
        }

        // Open Graph check
        $ogMissing = [];
        foreach (['og:title', 'og:image', 'og:description'] as $key) {
            if (empty($doc->openGraph[str_replace('og:', '', $key)] ?? '')) {
                $ogMissing[] = $key;
            }
        }
        if (!empty($ogMissing)) {
            $issues[] = "Missing OG tags: " . implode(', ', $ogMissing);
        }

        // Heading structure
        $headings = array_map(
            static fn ($h) => str_repeat('#', $h->level) . ' ' . $h->text,
            $result['metadata']->headers
        );

        // Image alt text check
        $imagesNoAlt = array_map(
            static fn ($img) => $img->src,
            array_filter(
                $result['metadata']->images,
                static fn ($img) => !$img->alt || strlen($img->alt) === 0
            )
        );
        if (!empty($imagesNoAlt)) {
            $issues[] = count($imagesNoAlt) . " images missing alt text";
        }

        // External links
        $externalLinks = array_map(
            static fn ($link) => $link->href,
            array_filter(
                $result['metadata']->links,
                static fn ($link) => $link->linkType === 'external'
            )
        );

        return [
            'title_missing' => $titleMissing,
            'description_missing' => $descMissing,
            'og_missing' => $ogMissing,
            'headings' => $headings,
            'images_missing_alt' => $imagesNoAlt,
            'external_links' => $externalLinks,
            'issues' => $issues,
        ];
    }
}

// Usage
$audit = new SeoAudit();
$report = $audit->audit($pageHtml);

if (!empty($report['issues'])) {
    echo "SEO Issues Found:\n";
    foreach ($report['issues'] as $issue) {
        echo "  - $issue\n";
    }
}
```

#### Example 2: Table of Contents Generator

```php
<?php

declare(strict_types=1);

use HtmlToMarkdown\Service\Converter;

class TableOfContentsGenerator
{
    private readonly Converter $converter;

    public function __construct()
    {
        $this->converter = Converter::create();
    }

    public function generate(string $html): string
    {
        $result = $this->converter->convertWithMetadata(
            $html,
            null,
            ['extract_headers' => true]
        );

        $toc = "## Table of Contents\n\n";
        $lastLevel = 0;

        foreach ($result['metadata']->headers as $header) {
            if ($header->level < 2) {
                continue;  // Skip h1
            }

            $indent = str_repeat('  ', $header->level - 2);
            $id = $header->id ?: $this->slugify($header->text);

            $toc .= $indent . "- [{$header->text}](#{$id})\n";
        }

        return $toc;
    }

    private function slugify(string $text): string
    {
        return strtolower(
            preg_replace('/[^a-z0-9]+/', '-', trim($text)) ?? ''
        );
    }
}

// Usage
$generator = new TableOfContentsGenerator();
$toc = $generator->generate($html);
echo $toc;
```

#### Example 3: Content Extractor with Asset Tracking

```php
<?php

declare(strict_types=1);

use HtmlToMarkdown\Service\Converter;
use HtmlToMarkdown\Config\ConversionOptions;

class ContentExtractor
{
    private readonly Converter $converter;

    public function __construct()
    {
        $this->converter = Converter::create();
    }

    /**
     * @return array{
     *     markdown: string,
     *     title: string|null,
     *     description: string|null,
     *     assets: array{
     *         images: list<array{src: string, alt: string|null}>,
     *         external_links: list<array{href: string, text: string}>,
     *     },
     * }
     */
    public function extract(string $html): array
    {
        $result = $this->converter->convertWithMetadata(
            $html,
            new ConversionOptions(headingStyle: 'Atx'),
            [
                'extract_links' => true,
                'extract_images' => true,
            ]
        );

        // Collect images
        $images = array_map(
            static fn ($img) => [
                'src' => $img->src,
                'alt' => $img->alt,
            ],
            $result['metadata']->images
        );

        // Collect external links (deduplicated)
        $externalLinks = array_values(
            array_unique(
                array_map(
                    static fn ($link) => [
                        'href' => $link->href,
                        'text' => $link->text,
                    ],
                    array_filter(
                        $result['metadata']->links,
                        static fn ($link) => $link->linkType === 'external'
                    )
                ),
                SORT_REGULAR
            )
        );

        return [
            'markdown' => $result['markdown'],
            'title' => $result['metadata']->document->title,
            'description' => $result['metadata']->document->description,
            'assets' => [
                'images' => $images,
                'external_links' => $externalLinks,
            ],
        ];
    }
}

// Usage
$extractor = new ContentExtractor();
$extracted = $extractor->extract($html);

file_put_contents('output.md', $extracted['markdown']);
file_put_contents('assets.json', json_encode($extracted['assets'], JSON_PRETTY_PRINT));
```

### Error Handling

Metadata extraction is type-safe. PHPStan validates all accesses. Conversion errors are wrapped in `ConversionFailed` exceptions:

```php
<?php

use HtmlToMarkdown\Service\Converter;
use HtmlToMarkdown\Exception\ConversionFailed;
use HtmlToMarkdown\Exception\ExtensionNotLoaded;

$converter = Converter::create();

try {
    $result = $converter->convertWithMetadata($html);
} catch (ExtensionNotLoaded $e) {
    // Extension not installed or enabled
    echo "Error: {$e->getMessage()}\n";
} catch (ConversionFailed $e) {
    // Rust conversion error (malformed HTML, Rust panic, etc.)
    echo "Conversion failed: {$e->getMessage()}\n";
}
```

Inputs that look like binary data (e.g., PDF bytes cast to a string) raise `ConversionFailed` with an `Invalid input`
message.

### Type Safety (PHPStan)

All metadata value objects are readonly with strict types:

```php
<?php

// PHPStan level: max (inferred from packages/php/phpstan.neon)

$result = $converter->convertWithMetadata($html);

// Type-safe property access (no possibility of null pointer):
/** @var \HtmlToMarkdown\Value\ExtendedMetadata */
$metadata = $result['metadata'];

// Each property is typed and readonly
/** @var string|null */
$title = $metadata->document->title;

/** @var list<\HtmlToMarkdown\Value\LinkMetadata> */
$links = $metadata->links;

// Iterate with full type knowledge
foreach ($links as $link) {
    // IDE autocomplete, PHPStan checking all accesses
    echo $link->href;  // string
    echo $link->text;  // string
    echo implode(', ', $link->rel);  // list<string>
}
```

No `Any` types, no casts, no `@var` suppression needed.

### Performance

Metadata extraction is zero-copy where possible. The Rust core parses structure once and returns all metadata in a single pass alongside Markdown generation.

## Testing and quality

```bash
composer run lint   # phpstan (level max) + php-cs-fixer dry-run
composer run format # php-cs-fixer auto-fix
composer run test   # builds the extension and runs PHPUnit
```

The test runner compiles the Rust extension into `target/release` and loads it
when executing PHPUnit. Tests cover conversion parity, option parsing, warning
handling, and inline image extraction.

## Troubleshooting

- **Extension not found**: build with `cargo build -p html-to-markdown-php --release`
  and ensure the resulting library resides in `target/release`. Update
  `php.ini` to include `extension=html_to_markdown`.
- **Missing Rust toolchain**: install Rust via `rustup` and ensure `cargo` is on
  the `PATH`.
- **PIE install copies to `/html_to_markdown.so`**: set the extension dir
  explicitly, e.g. `PHP_EXTENSION_DIR=$(php-config --extension-dir) pie install goldziher/html-to-markdown`
  (PIE then writes to that directory, and you can enable via `extension=html_to_markdown`).
- **Composer permissions**: run Composer with `--no-interaction --no-progress`
  in CI to avoid prompts.

## Contributing

1. Install dependencies (`composer install`) and build the extension
   (`cargo build -p html-to-markdown-php --release`).
2. Run `composer run lint` and `composer run test`.
3. Follow the existing coding guidelines enforced by php-cs-fixer and phpstan.

## License

MIT © Na'aman Hirschfeld
