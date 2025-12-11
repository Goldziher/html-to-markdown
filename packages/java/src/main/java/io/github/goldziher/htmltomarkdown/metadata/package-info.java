/**
 * Metadata extraction and representation types for HTML-to-Markdown conversion.
 *
 * <p>This package provides Java record types for metadata extracted during HTML-to-Markdown
 * conversion, including:
 *
 * <ul>
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.MetadataExtraction} - Main result type
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.ExtendedMetadata} - Complete metadata
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.DocumentMetadata} - Document-level metadata
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.HeaderMetadata} - Heading information
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.LinkMetadata} - Hyperlink information
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.ImageMetadata} - Image information
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.StructuredData} - Structured data blocks
 * </ul>
 *
 * <p>Type enums:
 *
 * <ul>
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.LinkType} - Link classification
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.ImageType} - Image source type
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.TextDirection} - Text directionality
 *   <li>{@link io.github.goldziher.htmltomarkdown.metadata.StructuredDataType} - Structured data format
 * </ul>
 *
 * <h2>Example Usage</h2>
 *
 * <pre>{@code
 * import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
 * import io.github.goldziher.htmltomarkdown.metadata.MetadataExtraction;
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
package io.github.goldziher.htmltomarkdown.metadata;
