package io.github.goldziher.htmltomarkdown.metadata;

import com.fasterxml.jackson.annotation.JsonInclude;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Objects;

/**
 * Hyperlink metadata with categorization and attributes.
 *
 * <p>Represents {@code <a>} elements with parsed href values, text content, and link type
 * classification.
 *
 * @param href The href URL value
 * @param text Link text content (normalized, concatenated if mixed with elements)
 * @param title Optional title attribute (often shown as tooltip)
 * @param linkType Link type classification
 * @param rel Rel attribute values (e.g., "nofollow", "stylesheet", "canonical")
 * @param attributes Additional HTML attributes
 * @since 2.13.0
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public record LinkMetadata(
    String href,
    String text,
    String title,
    LinkType linkType,
    List<String> rel,
    Map<String, String> attributes) {

  /**
   * Construct a LinkMetadata record.
   *
   * @param href the href URL
   * @param text the link text
   * @param title the optional title
   * @param linkType the link type
   * @param rel the rel attribute values
   * @param attributes the additional attributes
   */
  public LinkMetadata {
    Objects.requireNonNull(href, "href cannot be null");
    Objects.requireNonNull(text, "text cannot be null");
    Objects.requireNonNull(rel, "rel cannot be null");
    Objects.requireNonNull(attributes, "attributes cannot be null");
  }

  /**
   * Classify a link based on href value.
   *
   * @param href The href attribute value
   * @return Appropriate LinkType based on protocol and content
   *     <p>For example, {@code LinkMetadata.classifyLink("#section")} returns {@code
   *     LinkType.ANCHOR}.
   */
  public static LinkType classifyLink(String href) {
    if (href == null) {
      return LinkType.OTHER;
    }
    if (href.startsWith("#")) {
      return LinkType.ANCHOR;
    } else if (href.startsWith("mailto:")) {
      return LinkType.EMAIL;
    } else if (href.startsWith("tel:")) {
      return LinkType.PHONE;
    } else if (href.startsWith("http://") || href.startsWith("https://")) {
      return LinkType.EXTERNAL;
    } else if (href.startsWith("/") || href.startsWith("../") || href.startsWith("./")) {
      return LinkType.INTERNAL;
    }
    return LinkType.OTHER;
  }

  /**
   * Create a LinkMetadata for an external link.
   *
   * @param href the URL
   * @param text the link text
   * @return a new LinkMetadata with external link type
   */
  public static LinkMetadata external(String href, String text) {
    return new LinkMetadata(
        href, text, null, LinkType.EXTERNAL, Collections.emptyList(), Collections.emptyMap());
  }

  /**
   * Create a LinkMetadata for an internal link.
   *
   * @param href the relative URL
   * @param text the link text
   * @return a new LinkMetadata with internal link type
   */
  public static LinkMetadata internal(String href, String text) {
    return new LinkMetadata(
        href, text, null, LinkType.INTERNAL, Collections.emptyList(), Collections.emptyMap());
  }

  /**
   * Create a LinkMetadata for an email link.
   *
   * @param email the email address
   * @param text the link text
   * @return a new LinkMetadata with email link type
   */
  public static LinkMetadata email(String email, String text) {
    return new LinkMetadata(
        "mailto:" + email,
        text,
        null,
        LinkType.EMAIL,
        Collections.emptyList(),
        Collections.emptyMap());
  }
}
