package dev.kreuzberg.htmltomarkdown.visitor;

import java.util.Objects;

/**
 * Immutable representation of an HTML attribute pair.
 *
 * <p>Used in NodeContext to represent HTML attributes. Both key and value are non-null strings
 * (though value may be empty).
 *
 * @param key the attribute name (never null)
 * @param value the attribute value (never null, may be empty)
 * @since 2.17.0
 */
public record Attribute(String key, String value) {

  /**
   * Create a new Attribute with validation.
   *
   * @param key the attribute name
   * @param value the attribute value
   * @throws NullPointerException if key or value is null
   */
  public Attribute {
    Objects.requireNonNull(key, "Attribute key cannot be null");
    Objects.requireNonNull(value, "Attribute value cannot be null");
  }

  /**
   * Check if this attribute has an empty value.
   *
   * @return true if the value is empty
   */
  public boolean isEmpty() {
    return value.isEmpty();
  }

  /**
   * Return a string representation of this attribute in HTML syntax.
   *
   * @return HTML representation (e.g., "class=\"container\"")
   */
  @Override
  public String toString() {
    if (isEmpty()) {
      return key;
    }
    return key + "=\"" + value + "\"";
  }
}
