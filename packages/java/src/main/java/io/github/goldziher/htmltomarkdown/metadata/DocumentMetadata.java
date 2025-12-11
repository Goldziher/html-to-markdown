package io.github.goldziher.htmltomarkdown.metadata;

import com.fasterxml.jackson.annotation.JsonInclude;
import java.util.Map;
import java.util.Objects;

/**
 * Document-level metadata extracted from {@code <head>} and top-level elements.
 *
 * <p>Contains all metadata typically used by search engines, social media platforms,
 * and browsers for document indexing and presentation.
 *
 * @param title Document title from {@code <title>} tag
 * @param description Document description from {@code <meta name="description">} tag
 * @param keywords Document keywords from {@code <meta name="keywords">} tag, split on commas
 * @param author Document author from {@code <meta name="author">} tag
 * @param canonicalUrl Canonical URL from {@code <link rel="canonical">} tag
 * @param baseHref Base URL from {@code <base href="">} tag for resolving relative URLs
 * @param language Document language from {@code lang} attribute
 * @param textDirection Document text direction from {@code dir} attribute
 * @param openGraph Open Graph metadata (og:* properties) for social media
 * @param twitterCard Twitter Card metadata (twitter:* properties)
 * @param metaTags Additional meta tags not covered by specific fields
 *
 * @since 2.13.0
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public record DocumentMetadata(
    String title,
    String description,
    java.util.List<String> keywords,
    String author,
    String canonicalUrl,
    String baseHref,
    String language,
    TextDirection textDirection,
    Map<String, String> openGraph,
    Map<String, String> twitterCard,
    Map<String, String> metaTags) {

  /**
   * Construct a DocumentMetadata record with all fields.
   *
   * @param title the document title
   * @param description the document description
   * @param keywords the list of keywords
   * @param author the document author
   * @param canonicalUrl the canonical URL
   * @param baseHref the base href
   * @param language the document language
   * @param textDirection the text direction
   * @param openGraph the open graph properties
   * @param twitterCard the twitter card properties
   * @param metaTags additional meta tags
   */
  public DocumentMetadata {
    Objects.requireNonNull(keywords, "keywords cannot be null");
    Objects.requireNonNull(openGraph, "openGraph cannot be null");
    Objects.requireNonNull(twitterCard, "twitterCard cannot be null");
    Objects.requireNonNull(metaTags, "metaTags cannot be null");
  }

  /**
   * Create a DocumentMetadata with minimal information.
   *
   * @param title the document title
   * @param description the document description
   * @return a new DocumentMetadata instance
   */
  public static DocumentMetadata of(String title, String description) {
    return new DocumentMetadata(
        title,
        description,
        java.util.Collections.emptyList(),
        null,
        null,
        null,
        null,
        null,
        java.util.Collections.emptyMap(),
        java.util.Collections.emptyMap(),
        java.util.Collections.emptyMap());
  }
}
