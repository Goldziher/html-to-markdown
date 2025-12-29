package dev.kreuzberg.htmltomarkdown.metadata;

/**
 * Image source classification for proper handling and processing.
 *
 * <p>Determines whether an image is embedded (data URI), inline SVG, external, or relative.
 *
 * @since 2.13.0
 */
public enum ImageType {
  /** Data URI embedded image (base64 or other encoding). */
  DATA_URI("data_uri"),

  /** Inline SVG element. */
  INLINE_SVG("inline_svg"),

  /** External image URL (http/https). */
  EXTERNAL("external"),

  /** Relative image path. */
  RELATIVE("relative");

  private final String value;

  ImageType(String value) {
    this.value = value;
  }

  /**
   * Get the string representation of this image type.
   *
   * @return the type string ("data_uri", "inline_svg", "external", or "relative")
   */
  public String getValue() {
    return value;
  }

  /**
   * Parse an image type from string value.
   *
   * @param s the type string
   * @return the {@code ImageType}
   * @throws IllegalArgumentException if the string is null or unknown
   */
  public static ImageType parse(String s) {
    if (s == null) {
      throw new IllegalArgumentException("ImageType string cannot be null");
    }
    return switch (s.toLowerCase()) {
      case "data_uri" -> DATA_URI;
      case "inline_svg" -> INLINE_SVG;
      case "external" -> EXTERNAL;
      case "relative" -> RELATIVE;
      default -> throw new IllegalArgumentException("Unknown ImageType: " + s);
    };
  }

  @Override
  public String toString() {
    return value;
  }
}
