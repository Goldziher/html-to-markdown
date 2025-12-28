package io.github.goldziher.htmltomarkdown.metadata;

import com.fasterxml.jackson.annotation.JsonInclude;
import java.util.Objects;

/**
 * Result of HTML-to-Markdown conversion with metadata extraction.
 *
 * <p>Contains both the converted markdown output and the extracted metadata from the HTML document.
 * Suitable for applications that need both the markdown content and document analysis information.
 *
 * @param markdown The converted Markdown string
 * @param metadata The extracted metadata from the HTML document
 * @since 2.13.0
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public record MetadataExtraction(String markdown, ExtendedMetadata metadata) {

  /**
   * Construct a MetadataExtraction record.
   *
   * @param markdown the markdown content
   * @param metadata the extracted metadata
   */
  public MetadataExtraction {
    Objects.requireNonNull(markdown, "markdown cannot be null");
    Objects.requireNonNull(metadata, "metadata cannot be null");
  }

  /**
   * Get the markdown content.
   *
   * @return the converted markdown string
   */
  public String getMarkdown() {
    return markdown;
  }

  /**
   * Get the extracted metadata.
   *
   * @return the metadata object
   */
  public ExtendedMetadata getMetadata() {
    return metadata;
  }

  /**
   * Get the document-level metadata.
   *
   * @return the document metadata
   */
  public DocumentMetadata getDocumentMetadata() {
    return metadata.document();
  }

  /**
   * Get the number of headers extracted.
   *
   * @return the header count
   */
  public int getHeaderCount() {
    return metadata.getHeaderCount();
  }

  /**
   * Get the number of links extracted.
   *
   * @return the link count
   */
  public int getLinkCount() {
    return metadata.getLinkCount();
  }

  /**
   * Get the number of images extracted.
   *
   * @return the image count
   */
  public int getImageCount() {
    return metadata.getImageCount();
  }

  /**
   * Get the markdown length.
   *
   * @return the number of characters in the markdown
   */
  public int getMarkdownLength() {
    return markdown.length();
  }
}
