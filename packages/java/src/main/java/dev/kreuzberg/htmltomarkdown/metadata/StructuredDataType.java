package dev.kreuzberg.htmltomarkdown.metadata;

/**
 * Structured data format type.
 *
 * <p>Identifies the schema/format used for structured data markup.
 *
 * @since 2.13.0
 */
public enum StructuredDataType {
  /** JSON-LD (JSON for Linking Data) script blocks. */
  JSON_LD("json_ld"),

  /** HTML5 Microdata attributes (itemscope, itemtype, itemprop). */
  MICRODATA("microdata"),

  /** RDF in Attributes (RDFa) markup. */
  RDFA("rdfa");

  private final String value;

  StructuredDataType(String value) {
    this.value = value;
  }

  /**
   * Get the string representation of this structured data type.
   *
   * @return the type string ("json_ld", "microdata", or "rdfa")
   */
  public String getValue() {
    return value;
  }

  /**
   * Parse a structured data type from string value.
   *
   * @param s the type string
   * @return the {@code StructuredDataType}, or null if invalid
   */
  public static StructuredDataType parse(String s) {
    if (s == null) {
      return null;
    }
    return switch (s.toLowerCase()) {
      case "json_ld" -> JSON_LD;
      case "microdata" -> MICRODATA;
      case "rdfa" -> RDFA;
      default -> null;
    };
  }

  @Override
  public String toString() {
    return value;
  }
}
