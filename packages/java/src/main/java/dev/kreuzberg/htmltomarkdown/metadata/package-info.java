/**
 * Metadata extraction and representation types for HTML-to-Markdown conversion.
 *
 * <p>This package provides Java record types for metadata extracted during HTML-to-Markdown
 * conversion, including:
 *
 * <ul>
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.MetadataExtraction} - Main result type
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.ExtendedMetadata} - Complete metadata
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.DocumentMetadata} - Document-level
 *       metadata
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.HeaderMetadata} - Heading information
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.LinkMetadata} - Hyperlink information
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.ImageMetadata} - Image information
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.StructuredData} - Structured data blocks
 * </ul>
 *
 * <p>Type enums:
 *
 * <ul>
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.LinkType} - Link classification
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.ImageType} - Image source type
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.TextDirection} - Text directionality
 *   <li>{@link dev.kreuzberg.htmltomarkdown.metadata.StructuredDataType} - Structured data
 *       format
 * </ul>
 *
 * <h2>Example Usage</h2>
 *
 * <pre>{@code
 * import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
 * import dev.kreuzberg.htmltomarkdown.metadata.MetadataExtraction;
 *
 * String html = "<html><head><title>My Page</title></head><body>"
 *     + "<h1>Welcome</h1>"
 *     + "<a href=\"https://example.com\">Example</a>"
 *     + "</body></html>";
 *
 * MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
 *
 * String markdown = result.getMarkdown();
 * String title = result.getDocumentMetadata().title();
 * int linkCount = result.getLinkCount();
 * }</pre>
 *
 * @since 2.13.0
 */
package dev.kreuzberg.htmltomarkdown.metadata;
