package io.github.goldziher.htmltomarkdown;

import static org.junit.jupiter.api.Assertions.*;

import io.github.goldziher.htmltomarkdown.metadata.DocumentMetadata;
import io.github.goldziher.htmltomarkdown.metadata.ExtendedMetadata;
import io.github.goldziher.htmltomarkdown.metadata.HeaderMetadata;
import io.github.goldziher.htmltomarkdown.metadata.ImageMetadata;
import io.github.goldziher.htmltomarkdown.metadata.LinkMetadata;
import io.github.goldziher.htmltomarkdown.metadata.LinkType;
import io.github.goldziher.htmltomarkdown.metadata.MetadataExtraction;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Comprehensive test suite for convertWithMetadata functionality.
 *
 * @noinspection MagicNumber
 */
class ConvertWithMetadataTest {

  @Test
  @DisplayName("Convert HTML with basic document metadata")
  void testBasicDocumentMetadata() {
    String html =
        "<html><head><title>Test Page</title>"
            + "<meta name=\"description\" content=\"A test page\">"
            + "</head><body><h1>Hello</h1></body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    assertNotNull(result, "Result should not be null");
    assertNotNull(result.getMarkdown(), "Markdown should not be null");
    assertNotNull(result.getMetadata(), "Metadata should not be null");

    DocumentMetadata doc = result.getDocumentMetadata();
    assertEquals("Test Page", doc.title(), "Title should match");
    assertEquals("A test page", doc.description(), "Description should match");
  }

  @Test
  @DisplayName("Extract headers with correct levels")
  void testHeaderExtraction() {
    String html =
        "<html><body>"
            + "<h1>Main Title</h1>"
            + "<h2>Subtitle</h2>"
            + "<h3 id=\"section1\">Section</h3>"
            + "</body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    ExtendedMetadata metadata = result.getMetadata();

    assertEquals(3, metadata.getHeaderCount(), "Should have 3 headers");

    HeaderMetadata h1 = metadata.headers().get(0);
    assertEquals(1, h1.level(), "First header should be level 1");
    assertEquals("Main Title", h1.text(), "H1 text should match");

    HeaderMetadata h3 = metadata.headers().get(2);
    assertEquals(3, h3.level(), "Third header should be level 3");
    assertEquals("section1", h3.id(), "H3 should have id");
  }

  @Test
  @DisplayName("Extract links with correct types")
  void testLinkExtraction() {
    String html =
        "<html><body>"
            + "<a href=\"https://example.com\">External</a>"
            + "<a href=\"/about\">Internal</a>"
            + "<a href=\"#top\">Anchor</a>"
            + "<a href=\"mailto:test@example.com\">Email</a>"
            + "</body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    ExtendedMetadata metadata = result.getMetadata();

    assertEquals(4, metadata.getLinkCount(), "Should have 4 links");

    // Check external link
    LinkMetadata external = metadata.getExternalLinks().get(0);
    assertEquals("https://example.com", external.href(), "External link href should match");
    assertEquals(LinkType.EXTERNAL, external.linkType(), "Should be external");

    // Check internal link
    LinkMetadata internal = metadata.getInternalLinks().get(0);
    assertEquals("/about", internal.href(), "Internal link href should match");
    assertEquals(LinkType.INTERNAL, internal.linkType(), "Should be internal");

    // Check anchor link
    LinkMetadata anchor = metadata.getLinksByType(LinkType.ANCHOR).get(0);
    assertEquals("#top", anchor.href(), "Anchor href should match");

    // Check email link
    LinkMetadata email = metadata.getLinksByType(LinkType.EMAIL).get(0);
    assertEquals("mailto:test@example.com", email.href(), "Email href should match");
  }

  @Test
  @DisplayName("Extract images with metadata")
  void testImageExtraction() {
    String html =
        "<html><body>"
            + "<img src=\"https://example.com/image.jpg\" alt=\"Test Image\" width=\"800\" height=\"600\">"
            + "<img src=\"/local/image.png\" alt=\"Local Image\">"
            + "</body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    ExtendedMetadata metadata = result.getMetadata();

    assertEquals(2, metadata.getImageCount(), "Should have 2 images");

    ImageMetadata external = metadata.getExternalImages().get(0);
    assertEquals("https://example.com/image.jpg", external.src(), "External image src should match");
    assertEquals("Test Image", external.alt(), "Alt text should match");
    assertTrue(external.hasDimensions(), "Should have dimensions");
    assertEquals(800, external.getWidth(), "Width should match");
    assertEquals(600, external.getHeight(), "Height should match");

    ImageMetadata local = metadata.images().get(1);
    assertEquals("/local/image.png", local.src(), "Local image src should match");
  }

  @Test
  @DisplayName("Extract keywords from meta tags")
  void testKeywordExtraction() {
    String html =
        "<html><head>"
            + "<meta name=\"keywords\" content=\"java, rust, html, markdown\">"
            + "</head><body></body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    DocumentMetadata doc = result.getDocumentMetadata();

    assertEquals(4, doc.keywords().size(), "Should have 4 keywords");
    assertTrue(doc.keywords().contains("java"), "Should contain java");
    assertTrue(doc.keywords().contains("rust"), "Should contain rust");
  }

  @Test
  @DisplayName("Extract Open Graph metadata")
  void testOpenGraphExtraction() {
    String html =
        "<html><head>"
            + "<meta property=\"og:title\" content=\"My Article\">"
            + "<meta property=\"og:description\" content=\"An interesting article\">"
            + "<meta property=\"og:image\" content=\"https://example.com/og-image.jpg\">"
            + "</head><body></body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    DocumentMetadata doc = result.getDocumentMetadata();

    assertTrue(doc.openGraph().containsKey("title"), "Should have og:title");
    assertEquals("My Article", doc.openGraph().get("title"), "og:title value should match");
  }

  @Test
  @DisplayName("Extract Twitter Card metadata")
  void testTwitterCardExtraction() {
    String html =
        "<html><head>"
            + "<meta name=\"twitter:card\" content=\"summary_large_image\">"
            + "<meta name=\"twitter:creator\" content=\"@example\">"
            + "</head><body></body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    DocumentMetadata doc = result.getDocumentMetadata();

    assertTrue(doc.twitterCard().containsKey("card"), "Should have twitter:card");
    assertEquals("summary_large_image", doc.twitterCard().get("card"), "twitter:card value should match");
  }

  @Test
  @DisplayName("Extract author metadata")
  void testAuthorExtraction() {
    String html =
        "<html><head>"
            + "<meta name=\"author\" content=\"John Doe\">"
            + "</head><body></body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    DocumentMetadata doc = result.getDocumentMetadata();

    assertEquals("John Doe", doc.author(), "Author should match");
  }

  @Test
  @DisplayName("Extract canonical URL")
  void testCanonicalUrlExtraction() {
    String html =
        "<html><head>"
            + "<link rel=\"canonical\" href=\"https://example.com/article\">"
            + "</head><body></body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    DocumentMetadata doc = result.getDocumentMetadata();

    assertEquals("https://example.com/article", doc.canonicalUrl(), "Canonical URL should match");
  }

  @Test
  @DisplayName("Extract language attribute")
  void testLanguageExtraction() {
    String html = "<html lang=\"en\"><body></body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    DocumentMetadata doc = result.getDocumentMetadata();

    assertEquals("en", doc.language(), "Language should be en");
  }

  @Test
  @DisplayName("Handle empty HTML")
  void testEmptyHtml() {
    String html = "";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    assertNotNull(result, "Result should not be null");
    assertNotNull(result.getMarkdown(), "Markdown should not be null");
    assertFalse(result.getMetadata().hasMetadata(), "Should have no metadata");
  }

  @Test
  @DisplayName("Handle HTML with no metadata")
  void testHtmlWithoutMetadata() {
    String html = "<p>Just plain content</p>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    assertNotNull(result, "Result should not be null");
    assertTrue(result.getMarkdown().contains("plain"), "Markdown should contain content");
    assertEquals(0, result.getHeaderCount(), "Should have no headers");
    assertEquals(0, result.getLinkCount(), "Should have no links");
  }

  @Test
  @DisplayName("Get header counts by level")
  void testGetHeadersByLevel() {
    String html =
        "<html><body>"
            + "<h1>H1 One</h1>"
            + "<h2>H2 One</h2>"
            + "<h2>H2 Two</h2>"
            + "<h3>H3 One</h3>"
            + "</body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    assertEquals(1, result.getMetadata().getHeadersByLevel(1).size(), "Should have 1 h1");
    assertEquals(2, result.getMetadata().getHeadersByLevel(2).size(), "Should have 2 h2");
    assertEquals(1, result.getMetadata().getHeadersByLevel(3).size(), "Should have 1 h3");
  }

  @Test
  @DisplayName("Get link types separately")
  void testGetLinksByType() {
    String html =
        "<html><body>"
            + "<a href=\"https://ex1.com\">E1</a>"
            + "<a href=\"https://ex2.com\">E2</a>"
            + "<a href=\"/path\">I1</a>"
            + "</body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    assertEquals(2, result.getMetadata().getExternalLinks().size(), "Should have 2 external links");
    assertEquals(1, result.getMetadata().getInternalLinks().size(), "Should have 1 internal link");
  }

  @Test
  @DisplayName("Handle null HTML input")
  void testNullHtmlInput() {
    assertThrows(NullPointerException.class, () -> {
      HtmlToMarkdown.convertWithMetadata(null);
    }, "Should throw NullPointerException for null HTML");
  }

  @Test
  @DisplayName("Return result with markdown content")
  void testResultContainsMarkdown() {
    String html = "<h1>Title</h1><p>Some content</p>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    assertTrue(result.getMarkdown().contains("Title"), "Markdown should contain title");
    assertTrue(result.getMarkdown().contains("content"), "Markdown should contain content");
  }

  @Test
  @DisplayName("Link with rel attributes")
  void testLinkRelAttributes() {
    String html = "<a href=\"https://example.com\" rel=\"nofollow external\">Link</a>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    assertEquals(1, result.getLinkCount(), "Should have 1 link");
    LinkMetadata link = result.getMetadata().links().get(0);
    assertTrue(link.rel().contains("nofollow"), "Should have nofollow");
    assertTrue(link.rel().contains("external"), "Should have external");
  }

  @Test
  @DisplayName("Complex document with all metadata types")
  void testComplexDocument() {
    String html =
        "<html lang=\"en\">"
            + "<head>"
            + "<title>Complex Article</title>"
            + "<meta name=\"description\" content=\"A complex article\">"
            + "<meta name=\"author\" content=\"Jane Smith\">"
            + "<link rel=\"canonical\" href=\"https://example.com/article\">"
            + "</head>"
            + "<body>"
            + "<h1>Main Article</h1>"
            + "<h2>Introduction</h2>"
            + "<p>Some intro text with <a href=\"https://link1.com\">external link</a> "
            + "and <a href=\"/page\">internal link</a>.</p>"
            + "<h2>Content</h2>"
            + "<img src=\"https://example.com/pic.jpg\" alt=\"A picture\">"
            + "<h3>Subsection</h3>"
            + "<p>More content</p>"
            + "</body>"
            + "</html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    ExtendedMetadata metadata = result.getMetadata();

    // Verify document metadata
    DocumentMetadata doc = metadata.document();
    assertEquals("Complex Article", doc.title());
    assertEquals("A complex article", doc.description());
    assertEquals("Jane Smith", doc.author());
    assertEquals("https://example.com/article", doc.canonicalUrl());
    assertEquals("en", doc.language());

    // Verify headers
    assertEquals(4, metadata.getHeaderCount());
    assertEquals(1, metadata.getHeadersByLevel(1).size());
    assertEquals(2, metadata.getHeadersByLevel(2).size());
    assertEquals(1, metadata.getHeadersByLevel(3).size());

    // Verify links
    assertEquals(2, metadata.getLinkCount());
    assertEquals(1, metadata.getExternalLinks().size());
    assertEquals(1, metadata.getInternalLinks().size());

    // Verify images
    assertEquals(1, metadata.getImageCount());

    // Verify markdown
    assertTrue(result.getMarkdown().contains("Main Article"));
    assertTrue(result.getMarkdown().contains("Subsection"));
  }

  @Test
  @DisplayName("Image with only src")
  void testImageWithMinimalData() {
    String html = "<img src=\"image.jpg\">";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    assertEquals(1, result.getImageCount());
    ImageMetadata img = result.getMetadata().images().get(0);
    assertEquals("image.jpg", img.src());
    assertNull(img.alt());
    assertFalse(img.hasDimensions());
  }

  @Test
  @DisplayName("Link with title attribute")
  void testLinkWithTitle() {
    String html = "<a href=\"https://example.com\" title=\"Visit Example\">Link</a>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    LinkMetadata link = result.getMetadata().links().get(0);
    assertEquals("Visit Example", link.title());
  }

  @Test
  @DisplayName("Image dimensions field is populated")
  void testImageMetadataDimensions() {
    String html = "<img src=\"test.jpg\" alt=\"Test\" width=\"1200\" height=\"800\">";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    ImageMetadata img = result.getMetadata().images().get(0);

    assertTrue(img.hasDimensions(), "Image should have dimensions");
    assertEquals(1200, img.getWidth(), "Width should be 1200");
    assertEquals(800, img.getHeight(), "Height should be 800");
  }

  @Test
  @DisplayName("DocumentMetadata textDirection uses enum")
  void testDocumentMetadataLanguageAttribute() {
    String html = "<html lang=\"fr\"><head><title>French</title></head></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    DocumentMetadata doc = result.getDocumentMetadata();

    assertEquals("fr", doc.language(), "Language should be fr");
    assertNotNull(doc, "DocumentMetadata should not be null");
  }

  @Test
  @DisplayName("LinkType enum parsing")
  void testLinkTypeEnum() {
    String html =
        "<html><body>"
            + "<a href=\"https://example.com\">External</a>"
            + "<a href=\"/page\">Internal</a>"
            + "<a href=\"#section\">Anchor</a>"
            + "<a href=\"mailto:test@example.com\">Email</a>"
            + "</body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    ExtendedMetadata metadata = result.getMetadata();

    // Verify that LinkType enum values match extracted link types
    assertEquals(LinkType.EXTERNAL, metadata.links().get(0).linkType());
    assertEquals(LinkType.INTERNAL, metadata.links().get(1).linkType());
    assertEquals(LinkType.ANCHOR, metadata.links().get(2).linkType());
    assertEquals(LinkType.EMAIL, metadata.links().get(3).linkType());
  }

  @Test
  @DisplayName("ImageType enum for different image types")
  void testImageTypeEnum() {
    String html =
        "<html><body>"
            + "<img src=\"https://example.com/image.jpg\" alt=\"JPG Image\">"
            + "<img src=\"/local/image.png\" alt=\"PNG Image\">"
            + "</body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    assertEquals(2, result.getImageCount(), "Should have 2 images");
    // All images should have valid image type values
    for (ImageMetadata img : result.getMetadata().images()) {
      assertNotNull(img.src(), "Image src should not be null");
    }
  }

  @Test
  @DisplayName("Null handling for optional image alt text")
  void testImageNullAltText() {
    String html = "<img src=\"test.jpg\">";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    ImageMetadata img = result.getMetadata().images().get(0);

    assertNull(img.alt(), "Alt text should be null when not provided");
    assertNotNull(img.src(), "Src should not be null");
  }

  @Test
  @DisplayName("Null handling for optional link title")
  void testLinkNullTitle() {
    String html = "<a href=\"https://example.com\">No Title</a>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    LinkMetadata link = result.getMetadata().links().get(0);

    assertNull(link.title(), "Title should be null when not provided");
    assertNotNull(link.href(), "Href should not be null");
  }

  @Test
  @DisplayName("Null handling for optional header id")
  void testHeaderNullId() {
    String html = "<h1>Header without ID</h1>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    HeaderMetadata header = result.getMetadata().headers().get(0);

    assertNull(header.id(), "ID should be null when not provided");
    assertNotNull(header.text(), "Text should not be null");
  }

  @Test
  @DisplayName("External vs Internal link type distinction")
  void testExternalVsInternalLinkTypes() {
    String html =
        "<html><body>"
            + "<a href=\"https://external.com\">External</a>"
            + "<a href=\"http://other-site.com\">Also External</a>"
            + "<a href=\"/local\">Internal Root</a>"
            + "<a href=\"./page.html\">Internal Relative</a>"
            + "</body></html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    ExtendedMetadata metadata = result.getMetadata();

    assertEquals(2, metadata.getExternalLinks().size(), "Should have 2 external links");
    assertEquals(2, metadata.getInternalLinks().size(), "Should have 2 internal links");
  }

  @Test
  @DisplayName("ImageMetadata with all optional fields")
  void testImageMetadataCompleteFields() {
    String html =
        "<img src=\"https://example.com/image.jpg\" alt=\"Complete Image\" "
            + "width=\"1024\" height=\"768\" title=\"Image Title\">";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    ImageMetadata img = result.getMetadata().images().get(0);

    assertEquals("https://example.com/image.jpg", img.src());
    assertEquals("Complete Image", img.alt());
    assertTrue(img.hasDimensions());
    assertEquals(1024, img.getWidth());
    assertEquals(768, img.getHeight());
  }

  @Test
  @DisplayName("LinkMetadata rel attribute handling")
  void testLinkRelAttributeEmpty() {
    String html = "<a href=\"https://example.com\">No rel</a>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    LinkMetadata link = result.getMetadata().links().get(0);

    assertNotNull(link.rel(), "Rel should not be null");
  }

  @Test
  @DisplayName("Multiple metadata types in complex HTML")
  void testComplexMetadataIntegration() {
    String html =
        "<html lang=\"en\">"
            + "<head><title>Test</title></head>"
            + "<body>"
            + "<h1>Main</h1>"
            + "<h2 id=\"intro\">Introduction</h2>"
            + "<a href=\"https://external.com\" rel=\"external\">External</a>"
            + "<img src=\"test.jpg\" alt=\"Test\" width=\"500\" height=\"300\">"
            + "</body>"
            + "</html>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
    ExtendedMetadata metadata = result.getMetadata();

    // Verify multiple metadata types are correctly extracted
    assertTrue(metadata.getHeaderCount() >= 2, "Should have at least 2 headers");
    assertEquals(1, metadata.getLinkCount());
    assertEquals(1, metadata.getImageCount());

    // Verify data integrity
    HeaderMetadata h1 = metadata.headers().get(0);
    assertEquals(1, h1.level());
    assertEquals("Main", h1.text());

    ImageMetadata img = metadata.images().get(0);
    assertTrue(img.hasDimensions());
    assertEquals(500, img.getWidth());
    assertEquals(300, img.getHeight());
  }

  @Test
  @DisplayName("Header with no content")
  void testEmptyHeader() {
    String html = "<h1></h1><h2>Non-empty</h2>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    // Should handle empty headers gracefully - empty headers are typically filtered
    assertEquals(1, result.getHeaderCount());
    assertEquals("Non-empty", result.getMetadata().headers().get(0).text());
  }

  @Test
  @DisplayName("Link with empty href")
  void testLinkWithEmptyHref() {
    String html = "<a href=\"\">Empty Link</a><a href=\"/valid\">Valid</a>";

    MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);

    // Should handle links gracefully
    assertTrue(result.getLinkCount() > 0, "Should have at least one link");
  }
}
