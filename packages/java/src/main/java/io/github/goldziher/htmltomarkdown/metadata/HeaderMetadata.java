package io.github.goldziher.htmltomarkdown.metadata;

import com.fasterxml.jackson.annotation.JsonInclude;
import java.util.Objects;

/**
 * Header element metadata with hierarchy tracking.
 *
 * <p>Captures heading elements (h1-h6) with their text content, identifiers,
 * and position in the document structure.
 *
 * @param level Header level: 1 (h1) through 6 (h6)
 * @param text Normalized text content of the header
 * @param id HTML id attribute if present
 * @param depth Document tree depth at the header element
 * @param htmlOffset Byte offset in original HTML document
 *
 * @since 2.13.0
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public record HeaderMetadata(int level, String text, String id, int depth, int htmlOffset) {

  private static final int MIN_HEADER_LEVEL = 1;
  private static final int MAX_HEADER_LEVEL = 6;
  private static final int H3_LEVEL = 3;

  /**
   * Construct a HeaderMetadata record.
   *
   * @param level the header level (1-6)
   * @param text the header text content
   * @param id the optional HTML id attribute
   * @param depth the document tree depth
   * @param htmlOffset the byte offset in the HTML document
   */
  public HeaderMetadata {
    Objects.requireNonNull(text, "text cannot be null");
  }

  /**
   * Validate that the header level is within valid range (1-6).
   *
   * @return true if level is 1-6, false otherwise
   *
   * @apiNote For example, a header with level 3 returns true.
   */
  public boolean isValid() {
    return level >= MIN_HEADER_LEVEL && level <= MAX_HEADER_LEVEL;
  }

  /**
   * Create a HeaderMetadata for an h1 element.
   *
   * @param text the header text
   * @return a new HeaderMetadata with level 1
   */
  public static HeaderMetadata h1(String text) {
    return new HeaderMetadata(MIN_HEADER_LEVEL, text, null, 0, 0);
  }

  /**
   * Create a HeaderMetadata for an h2 element.
   *
   * @param text the header text
   * @return a new HeaderMetadata with level 2
   */
  public static HeaderMetadata h2(String text) {
    return new HeaderMetadata(2, text, null, 0, 0);
  }

  /**
   * Create a HeaderMetadata for an h3 element.
   *
   * @param text the header text
   * @return a new HeaderMetadata with level 3
   */
  public static HeaderMetadata h3(String text) {
    return new HeaderMetadata(H3_LEVEL, text, null, 0, 0);
  }
}
