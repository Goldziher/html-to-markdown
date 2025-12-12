<?php

declare(strict_types=1);

namespace HtmlToMarkdown;

use HtmlToMarkdown\Config\ConversionOptions;
use HtmlToMarkdown\Config\InlineImageConfig;
use HtmlToMarkdown\Value\ExtendedMetadata;
use HtmlToMarkdown\Value\InlineImageExtraction;

/**
 * @phpstan-import-type ConversionOptionsInput from \HtmlToMarkdown\Config\ConversionOptions
 * @phpstan-import-type InlineImageConfigInput from \HtmlToMarkdown\Config\InlineImageConfig
 */

/**
 * @param ConversionOptions|array<string, mixed>|null $options
 * @phpstan-param ConversionOptions|array<string, mixed>|null $options
 */
function convert(string $html, ConversionOptions|array|null $options = null): string
{
    return HtmlToMarkdown::convert($html, $options);
}

/**
 * @param ConversionOptions|array<string, mixed>|null $options
 * @param InlineImageConfig|array<string, mixed>|null $config
 * @phpstan-param ConversionOptions|array<string, mixed>|null $options
 * @phpstan-param InlineImageConfig|array<string, mixed>|null $config
 */
function convert_with_inline_images(
    string $html,
    ConversionOptions|array|null $options = null,
    InlineImageConfig|array|null $config = null,
): InlineImageExtraction {
    return HtmlToMarkdown::convertWithInlineImages($html, $options, $config);
}

/**
 * Convert HTML to Markdown with comprehensive metadata extraction.
 *
 * Performs HTML-to-Markdown conversion while simultaneously extracting structured metadata
 * including document properties, headers, links, images, and structured data in a single pass.
 * Ideal for content analysis, SEO optimization, and document indexing workflows.
 *
 * @param string $html HTML string to convert. Line endings are normalized (CRLF -> LF).
 *
 * @param ConversionOptions|array<string, mixed>|null $options Optional conversion configuration.
 *   Can be a ConversionOptions instance or associative array with keys:
 *   - heading_style: string ("atx", "atx_closed", or "underlined", default: "underlined")
 *   - list_indent_type: string ("spaces" or "tabs", default: "spaces")
 *   - list_indent_width: int (default: 4)
 *   - bullets: string (default: "*+-")
 *   - strong_em_symbol: string (default: "*")
 *   - wrap: bool (default: false)
 *   - wrap_width: int (default: 80)
 *   - code_language: string (default: "")
 *   - escape_asterisks: bool (default: false)
 *   - escape_underscores: bool (default: false)
 *   - escape_misc: bool (default: false)
 *   - escape_ascii: bool (default: false)
 *   - And many more - see ConversionOptions documentation
 *
 * @param array<string, bool|int|string>|null $metadataConfig Optional metadata extraction configuration.
 *   Associative array controlling which metadata to extract:
 *   - extract_headers: bool (Extract h1-h6 elements, default: true)
 *   - extract_links: bool (Extract hyperlinks, default: true)
 *   - extract_images: bool (Extract image elements, default: true)
 *   - extract_structured_data: bool (Extract JSON-LD/Microdata/RDFa, default: true)
 *   - max_structured_data_size: int (Size limit in bytes, default: 1000000)
 *
 * @return array{markdown: string, metadata: ExtendedMetadata} Associative array with keys:
 *   - "markdown": string - The converted Markdown output
 *   - "metadata": ExtendedMetadata - Comprehensive metadata object with properties:
 *
 *     * document: DocumentMetadata - Document-level metadata:
 *       - title?: string - From <title> tag
 *       - description?: string - From <meta name="description">
 *       - keywords: string[] - From <meta name="keywords">
 *       - author?: string - From <meta name="author">
 *       - language?: string - From lang attribute (e.g., "en")
 *       - text_direction?: string - "ltr", "rtl", or "auto"
 *       - canonical_url?: string - From <link rel="canonical">
 *       - base_href?: string - From <base href="">
 *       - open_graph: array<string, string> - Open Graph properties (og:* meta tags)
 *       - twitter_card: array<string, string> - Twitter Card properties (twitter:* meta tags)
 *       - meta_tags: array<string, string> - Other meta tags
 *
 *     * headers: HeaderMetadata[] - Heading elements:
 *       - level: int (1-6)
 *       - text: string - Header text content
 *       - id?: string - HTML id attribute
 *       - depth: int - Tree nesting depth
 *       - html_offset: int - Byte offset in original HTML
 *
 *     * links: LinkMetadata[] - Hyperlinks:
 *       - href: string - Link URL
 *       - text: string - Link text content
 *       - title?: string - Title attribute
 *       - link_type: string - "anchor", "internal", "external", "email", "phone", or "other"
 *       - rel: string[] - Rel attribute values
 *       - attributes: array<string, string> - Additional HTML attributes
 *
 *     * images: ImageMetadata[] - Image elements:
 *       - src: string - Image source URL or data URI
 *       - alt?: string - Alt text for accessibility
 *       - title?: string - Title attribute
 *       - dimensions?: [int, int] - [width, height] if available
 *       - image_type: string - "data_uri", "external", "relative", or "inline_svg"
 *       - attributes: array<string, string> - Additional HTML attributes
 *
 *     * structured_data: StructuredData[] - Structured data blocks:
 *       - data_type: string - "json_ld", "microdata", or "rdfa"
 *       - raw_json: string - Raw JSON content
 *       - schema_type?: string - Schema type (e.g., "Article", "Event")
 *
 * @throws \Throwable If HTML parsing fails, configuration is invalid, or conversion fails (thrown by the native extension)
 *
 * @example Basic usage
 * ```php
 * $html = <<<HTML
 *   <html lang="en">
 *     <head>
 *       <title>My Article</title>
 *       <meta name="description" content="A great read">
 *     </head>
 *     <body>
 *       <h1 id="intro">Introduction</h1>
 *       <p>Visit <a href="https://example.com">our site</a></p>
 *       <img src="photo.jpg" alt="Beautiful landscape">
 *     </body>
 *   </html>
 * HTML;
 *
 * $result = convert_with_metadata($html);
 * $markdown = $result['markdown'];
 * $metadata = $result['metadata'];
 *
 * echo $metadata->document->title;        // "My Article"
 * echo $metadata->document->language;     // "en"
 * echo count($metadata->headers);         // 1
 * echo $metadata->headers[0]->text;       // "Introduction"
 * echo count($metadata->links);           // 1
 * echo count($metadata->images);          // 1
 * ```
 *
 * @example With selective metadata extraction
 * ```php
 * $metadataConfig = [
 *     'extract_headers' => true,
 *     'extract_links' => true,
 *     'extract_images' => false,           // Skip images
 *     'extract_structured_data' => false   // Skip structured data
 * ];
 *
 * $result = convert_with_metadata($html, null, $metadataConfig);
 * $metadata = $result['metadata'];
 *
 * assert(count($metadata->images) === 0);  // Images not extracted
 * assert(count($metadata->headers) > 0);   // Headers extracted
 * ```
 *
 * @example With custom conversion options
 * ```php
 * $options = [
 *     'heading_style' => 'atx',  // Use # H1, ## H2 style
 *     'wrap' => true,
 *     'wrap_width' => 80
 * ];
 *
 * $metadataConfig = [
 *     'extract_headers' => true,
 *     'extract_links' => true
 * ];
 *
 * $result = convert_with_metadata($html, $options, $metadataConfig);
 * // Markdown uses ATX-style headings and wraps at 80 characters
 * ```
 *
 * @see convert() Simple conversion without metadata
 * @see convert_with_inline_images() Extract inline images during conversion
 * @see ConversionOptions Configuration options documentation
 *
 * @phpstan-param ConversionOptions|array<string, mixed>|null $options
 * @return array{markdown: string, metadata: ExtendedMetadata}
 */
function convert_with_metadata(
    string $html,
    ConversionOptions|array|null $options = null,
    ?array $metadataConfig = null,
): array {
    return HtmlToMarkdown::convertWithMetadata($html, $options, $metadataConfig);
}
