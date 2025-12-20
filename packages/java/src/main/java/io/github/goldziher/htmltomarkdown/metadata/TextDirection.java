package io.github.goldziher.htmltomarkdown.metadata;

/**
 * Text directionality of document content.
 *
 * <p>Corresponds to the HTML {@code dir} attribute and {@code bdi} element directionality.
 *
 * @since 2.13.0
 */
public enum TextDirection {
  /** Left-to-right text flow (default for Latin scripts). */
  LEFT_TO_RIGHT("ltr"),

  /** Right-to-left text flow (Hebrew, Arabic, Urdu, etc.). */
  RIGHT_TO_LEFT("rtl"),

  /** Automatic directionality detection. */
  AUTO("auto");

  private final String value;

  TextDirection(String value) {
    this.value = value;
  }

  /**
   * Get the string representation of this direction.
   *
   * @return the direction string ("ltr", "rtl", or "auto")
   */
  public String getValue() {
    return value;
  }

  /**
   * Parse a text direction from string value.
   *
   * @param s the direction string ("ltr", "rtl", or "auto")
   * @return the {@code TextDirection}
   * @throws IllegalArgumentException if the string is null or unknown
   *
   * For example, {@code TextDirection.parse("ltr")} returns {@code TextDirection.LEFT_TO_RIGHT}.
   */
  public static TextDirection parse(String s) {
    if (s == null) {
      throw new IllegalArgumentException("TextDirection string cannot be null");
    }
    return switch (s.toLowerCase()) {
      case "ltr" -> LEFT_TO_RIGHT;
      case "rtl" -> RIGHT_TO_LEFT;
      case "auto" -> AUTO;
      default -> throw new IllegalArgumentException("Unknown TextDirection: " + s);
    };
  }

  @Override
  public String toString() {
    return value;
  }
}
