package io.github.goldziher.htmltomarkdown.metadata;

import com.fasterxml.jackson.annotation.JsonInclude;
import java.util.Collections;
import java.util.Map;
import java.util.Objects;

/**
 * Image metadata with source and dimensions.
 *
 * <p>Captures {@code <img>} elements and inline {@code <svg>} elements with metadata
 * for image analysis and optimization.
 *
 * @param src Image source (URL, data URI, or SVG content identifier)
 * @param alt Alternative text from alt attribute (for accessibility)
 * @param title Title attribute (often shown as tooltip)
 * @param dimensions Image dimensions as [width, height] if available
 * @param imageType Image type classification
 * @param attributes Additional HTML attributes
 *
 * @since 2.13.0
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public record ImageMetadata(
    String src, String alt, String title, int[] dimensions, ImageType imageType, Map<String, String> attributes) {

  /**
   * Construct an ImageMetadata record.
   *
   * @param src the image source
   * @param alt the alternative text
   * @param title the optional title
   * @param dimensions the optional dimensions as [width, height]
   * @param imageType the image type
   * @param attributes the additional attributes
   */
  public ImageMetadata {
    Objects.requireNonNull(src, "src cannot be null");
    Objects.requireNonNull(imageType, "imageType cannot be null");
    Objects.requireNonNull(attributes, "attributes cannot be null");
  }

  /**
   * Check if the image has dimensions specified.
   *
   * @return true if dimensions are present, false otherwise
   */
  public boolean hasDimensions() {
    return dimensions != null && dimensions.length == 2;
  }

  /**
   * Get the image width if available.
   *
   * @return the width in pixels, or -1 if not specified
   */
  public int getWidth() {
    return hasDimensions() ? dimensions[0] : -1;
  }

  /**
   * Get the image height if available.
   *
   * @return the height in pixels, or -1 if not specified
   */
  public int getHeight() {
    return hasDimensions() ? dimensions[1] : -1;
  }

  /**
   * Check if this is an external image URL.
   *
   * @return true if image type is EXTERNAL
   */
  public boolean isExternal() {
    return imageType == ImageType.EXTERNAL;
  }

  /**
   * Check if this is a data URI embedded image.
   *
   * @return true if image type is DATA_URI
   */
  public boolean isDataUri() {
    return imageType == ImageType.DATA_URI;
  }

  /**
   * Create an ImageMetadata for an external image.
   *
   * @param src the image URL
   * @param alt the alternative text
   * @return a new ImageMetadata with external type
   */
  public static ImageMetadata external(String src, String alt) {
    return new ImageMetadata(
        src,
        alt,
        null,
        null,
        ImageType.EXTERNAL,
        Collections.emptyMap());
  }

  /**
   * Create an ImageMetadata for a relative image.
   *
   * @param src the relative path
   * @param alt the alternative text
   * @return a new ImageMetadata with relative type
   */
  public static ImageMetadata relative(String src, String alt) {
    return new ImageMetadata(
        src,
        alt,
        null,
        null,
        ImageType.RELATIVE,
        Collections.emptyMap());
  }

  /**
   * Create an ImageMetadata for a data URI image.
   *
   * @param dataUri the data URI string
   * @param alt the alternative text
   * @return a new ImageMetadata with data_uri type
   */
  public static ImageMetadata dataUri(String dataUri, String alt) {
    return new ImageMetadata(
        dataUri,
        alt,
        null,
        null,
        ImageType.DATA_URI,
        Collections.emptyMap());
  }
}
