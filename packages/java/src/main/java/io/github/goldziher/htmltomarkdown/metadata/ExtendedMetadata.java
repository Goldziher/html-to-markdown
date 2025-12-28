package io.github.goldziher.htmltomarkdown.metadata;

import com.fasterxml.jackson.annotation.JsonInclude;
import java.util.Collections;
import java.util.List;
import java.util.Objects;

/**
 * Comprehensive metadata extraction result from HTML document.
 *
 * <p>Contains all extracted metadata types in a single structure, suitable for serialization and
 * transmission across language boundaries.
 *
 * @param document Document-level metadata (title, description, canonical, etc.)
 * @param headers Extracted header elements with hierarchy
 * @param links Extracted hyperlinks with type classification
 * @param images Extracted images with source and dimensions
 * @param structuredData Extracted structured data blocks
 * @since 2.13.0
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public record ExtendedMetadata(
    DocumentMetadata document,
    List<HeaderMetadata> headers,
    List<LinkMetadata> links,
    List<ImageMetadata> images,
    List<StructuredData> structuredData) {

  /**
   * Construct an ExtendedMetadata record.
   *
   * @param document the document metadata
   * @param headers the header metadata list
   * @param links the link metadata list
   * @param images the image metadata list
   * @param structuredData the structured data list
   */
  public ExtendedMetadata {
    Objects.requireNonNull(document, "document cannot be null");
    Objects.requireNonNull(headers, "headers cannot be null");
    Objects.requireNonNull(links, "links cannot be null");
    Objects.requireNonNull(images, "images cannot be null");
    Objects.requireNonNull(structuredData, "structuredData cannot be null");
  }

  /**
   * Get the number of header elements extracted.
   *
   * @return the header count
   */
  public int getHeaderCount() {
    return headers.size();
  }

  /**
   * Get the number of links extracted.
   *
   * @return the link count
   */
  public int getLinkCount() {
    return links.size();
  }

  /**
   * Get the number of images extracted.
   *
   * @return the image count
   */
  public int getImageCount() {
    return images.size();
  }

  /**
   * Get the number of structured data blocks extracted.
   *
   * @return the structured data count
   */
  public int getStructuredDataCount() {
    return structuredData.size();
  }

  /**
   * Check if any metadata was extracted.
   *
   * @return true if at least one metadata item exists
   */
  public boolean hasMetadata() {
    return !headers.isEmpty() || !links.isEmpty() || !images.isEmpty() || !structuredData.isEmpty();
  }

  /**
   * Get all headers with a specific level.
   *
   * @param level the header level (1-6)
   * @return list of headers at the specified level
   */
  public List<HeaderMetadata> getHeadersByLevel(int level) {
    return headers.stream().filter(h -> h.level() == level).toList();
  }

  /**
   * Get all links of a specific type.
   *
   * @param type the link type string
   * @return list of links of the specified type
   */
  public List<LinkMetadata> getLinksByType(String type) {
    LinkType linkType;
    try {
      linkType = LinkType.parse(type);
    } catch (IllegalArgumentException e) {
      return Collections.emptyList();
    }
    return links.stream().filter(l -> l.linkType() == linkType).toList();
  }

  /**
   * Get all links of a specific type.
   *
   * @param type the link type enum
   * @return list of links of the specified type
   */
  public List<LinkMetadata> getLinksByType(LinkType type) {
    return links.stream().filter(l -> l.linkType() == type).toList();
  }

  /**
   * Get all external links.
   *
   * @return list of external links
   */
  public List<LinkMetadata> getExternalLinks() {
    return getLinksByType(LinkType.EXTERNAL);
  }

  /**
   * Get all internal links.
   *
   * @return list of internal links
   */
  public List<LinkMetadata> getInternalLinks() {
    return getLinksByType(LinkType.INTERNAL);
  }

  /**
   * Get all images of a specific type.
   *
   * @param type the image type string
   * @return list of images of the specified type
   */
  public List<ImageMetadata> getImagesByType(String type) {
    ImageType imageType;
    try {
      imageType = ImageType.parse(type);
    } catch (IllegalArgumentException e) {
      return Collections.emptyList();
    }
    return images.stream().filter(i -> i.imageType() == imageType).toList();
  }

  /**
   * Get all images of a specific type.
   *
   * @param type the image type enum
   * @return list of images of the specified type
   */
  public List<ImageMetadata> getImagesByType(ImageType type) {
    return images.stream().filter(i -> i.imageType() == type).toList();
  }

  /**
   * Get all external images.
   *
   * @return list of external images
   */
  public List<ImageMetadata> getExternalImages() {
    return getImagesByType(ImageType.EXTERNAL);
  }

  /**
   * Create an empty ExtendedMetadata.
   *
   * @return a new ExtendedMetadata with all empty lists
   */
  public static ExtendedMetadata empty() {
    return new ExtendedMetadata(
        new DocumentMetadata(
            null,
            null,
            Collections.emptyList(),
            null,
            null,
            null,
            null,
            null,
            Collections.emptyMap(),
            Collections.emptyMap(),
            Collections.emptyMap()),
        Collections.emptyList(),
        Collections.emptyList(),
        Collections.emptyList(),
        Collections.emptyList());
  }
}
