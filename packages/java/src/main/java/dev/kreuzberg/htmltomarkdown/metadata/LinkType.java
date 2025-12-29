package dev.kreuzberg.htmltomarkdown.metadata;

/**
 * Link classification based on href value and document context.
 *
 * <p>Used to categorize links during extraction for filtering and analysis.
 *
 * @since 2.13.0
 */
public enum LinkType {
  /** Anchor link within same document (href starts with #). */
  ANCHOR("anchor"),

  /** Internal link within same domain. */
  INTERNAL("internal"),

  /** External link to different domain. */
  EXTERNAL("external"),

  /** Email link (mailto:). */
  EMAIL("email"),

  /** Phone link (tel:). */
  PHONE("phone"),

  /** Other protocol or unclassifiable. */
  OTHER("other");

  private final String value;

  LinkType(String value) {
    this.value = value;
  }

  /**
   * Get the string representation of this link type.
   *
   * @return the type string ("anchor", "internal", "external", "email", "phone", or "other")
   */
  public String getValue() {
    return value;
  }

  /**
   * Parse a link type from string value.
   *
   * @param s the type string
   * @return the {@code LinkType}
   * @throws IllegalArgumentException if the string is null or unknown
   */
  public static LinkType parse(String s) {
    if (s == null) {
      throw new IllegalArgumentException("LinkType string cannot be null");
    }
    return switch (s.toLowerCase()) {
      case "anchor" -> ANCHOR;
      case "internal" -> INTERNAL;
      case "external" -> EXTERNAL;
      case "email" -> EMAIL;
      case "phone" -> PHONE;
      case "other" -> OTHER;
      default -> throw new IllegalArgumentException("Unknown LinkType: " + s);
    };
  }

  @Override
  public String toString() {
    return value;
  }
}
