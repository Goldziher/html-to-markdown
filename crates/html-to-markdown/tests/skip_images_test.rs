#![allow(missing_docs)]

//! Tests for the `skip_images` functionality.
//!
//! This test suite verifies that the `skip_images` option correctly omits all `<img>` tags
//! from the markdown output when enabled, while preserving all other content.
//!
//! The `skip_images` option is useful for:
//! - Text-only extraction from HTML documents
//! - Filtering out visual content for accessibility or reduced bandwidth
//! - Converting image-heavy documents to plain text markdown

use html_to_markdown_rs::{ConversionOptions, convert};

#[test]
fn test_skip_images_enabled() {
    // Verify that when skip_images: true, all img tags are omitted
    let html = r#"<p>Here is an image:</p>
<img src="test.jpg" alt="Test Image" />
<p>And here is some text after.</p>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain text content
    assert!(result.contains("Here is an image"), "Should contain text before image");
    assert!(
        result.contains("And here is some text after"),
        "Should contain text after image"
    );

    // Should NOT contain the image markdown
    assert!(!result.contains("![Test Image]"), "Should not contain image markdown");
    assert!(!result.contains("test.jpg"), "Should not contain image URL");
}

#[test]
fn test_skip_images_disabled() {
    // Verify that when skip_images: false (default), images are converted to markdown
    let html = r#"<p>Here is an image:</p>
<img src="test.jpg" alt="Test Image" />
<p>And here is some text after.</p>"#;

    let options = ConversionOptions {
        skip_images: false,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain text content
    assert!(result.contains("Here is an image"), "Should contain text before image");
    assert!(
        result.contains("And here is some text after"),
        "Should contain text after image"
    );

    // Should contain the image markdown
    assert!(result.contains("![Test Image]"), "Should contain image markdown");
    assert!(result.contains("test.jpg"), "Should contain image URL");
}

#[test]
fn test_skip_images_default_behavior() {
    // Verify that default behavior (without specifying skip_images) includes images
    let html = r#"<img src="default.png" alt="Default Image" />"#;

    let result = convert(html, None).unwrap();

    // Default should be to include images (skip_images: false)
    assert!(result.contains("![Default Image]"), "Default should include images");
    assert!(result.contains("default.png"), "Default should include image URLs");
}

#[test]
fn test_skip_images_mixed_content() {
    // Test HTML with both images and other content to ensure only images are skipped
    let html = r#"<article>
<h1>Article Title</h1>
<p>Introduction paragraph.</p>
<img src="hero.jpg" alt="Hero Image" />
<h2>Section One</h2>
<p>Section content with <strong>bold text</strong> and <em>italic text</em>.</p>
<img src="section-image.png" alt="Section Image" />
<h2>Section Two</h2>
<p>More content here.</p>
<img src="footer-image.gif" alt="Footer Image" />
<footer>
  <p>Footer text with a <a href="https://example.com">link</a>.</p>
</footer>
</article>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain all text content
    assert!(result.contains("Article Title"), "Should contain heading");
    assert!(result.contains("Introduction paragraph"), "Should contain intro");
    assert!(result.contains("Section One"), "Should contain section heading");
    assert!(result.contains("Section content"), "Should contain section content");
    assert!(result.contains("bold text"), "Should contain bold text");
    assert!(result.contains("italic text"), "Should contain italic text");
    assert!(result.contains("Section Two"), "Should contain second section");
    assert!(result.contains("More content"), "Should contain more content");
    assert!(result.contains("Footer text"), "Should contain footer");
    assert!(result.contains("example.com"), "Should contain link");

    // Should NOT contain any images
    assert!(!result.contains("![Hero Image]"), "Should not contain hero image");
    assert!(!result.contains("hero.jpg"), "Should not contain hero image URL");
    assert!(!result.contains("![Section Image]"), "Should not contain section image");
    assert!(
        !result.contains("section-image.png"),
        "Should not contain section image URL"
    );
    assert!(!result.contains("![Footer Image]"), "Should not contain footer image");
    assert!(
        !result.contains("footer-image.gif"),
        "Should not contain footer image URL"
    );
}

#[test]
fn test_skip_images_with_base64_data_uri() {
    // Verify base64 data URI images are skipped
    let html = r#"<p>Before image</p>
<img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==" alt="Embedded PNG" />
<p>After image</p>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain text
    assert!(result.contains("Before image"), "Should contain text before image");
    assert!(result.contains("After image"), "Should contain text after image");

    // Should NOT contain base64 data or image markdown
    assert!(!result.contains("![Embedded PNG]"), "Should not contain base64 image");
    assert!(!result.contains("data:image"), "Should not contain data URI");
    assert!(!result.contains("iVBORw0KGgo"), "Should not contain base64 content");
}

#[test]
fn test_skip_images_with_external_urls() {
    // Verify external URL images are skipped
    let html = r#"<section>
<h1>Photo Gallery</h1>
<p>Check out these amazing photos:</p>
<img src="https://example.com/images/photo1.jpg" alt="Photo 1" />
<img src="https://cdn.example.org/photo2.png" alt="Photo 2" />
<img src="https://images.example.net/photo3.webp" alt="Photo 3" />
<p>Thanks for viewing!</p>
</section>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain text content
    assert!(result.contains("Photo Gallery"), "Should contain gallery heading");
    assert!(result.contains("Check out"), "Should contain intro text");
    assert!(result.contains("Thanks for viewing"), "Should contain closing text");

    // Should NOT contain any image references
    assert!(
        !result.contains("![Photo 1]"),
        "Should not contain first photo markdown"
    );
    assert!(
        !result.contains("![Photo 2]"),
        "Should not contain second photo markdown"
    );
    assert!(
        !result.contains("![Photo 3]"),
        "Should not contain third photo markdown"
    );
    assert!(!result.contains("example.com/images"), "Should not contain photo URLs");
    assert!(!result.contains("cdn.example.org"), "Should not contain CDN URLs");
}

#[test]
fn test_skip_images_preserves_alt_text_context() {
    // Ensure surrounding content is preserved correctly and not confused with alt text
    let html = r#"<div>
<p>The following image demonstrates our product:</p>
<img src="product.jpg" alt="Product Screenshot" />
<p>As you can see, this is how the interface looks.</p>
<img src="feature.png" alt="Feature Comparison Chart" />
<p>Our solution outperforms the competition.</p>
</div>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Verify all surrounding text is present and properly ordered
    assert!(result.contains("following image"), "Should contain introductory text");
    assert!(
        result.contains("how the interface looks"),
        "Should contain descriptive text"
    );
    assert!(
        result.contains("solution outperforms"),
        "Should contain concluding text"
    );

    // Verify images are not present (alt text is not included as text)
    assert!(!result.contains("Product Screenshot"), "Should not include alt text");
    assert!(
        !result.contains("Feature Comparison Chart"),
        "Should not include alt text"
    );
    assert!(!result.contains("product.jpg"), "Should not contain image URL");
    assert!(!result.contains("feature.png"), "Should not contain image URL");
}

#[test]
fn test_skip_images_inline_vs_block_images() {
    // Test skipping both inline and block-level images
    let html = r#"<p>Start of paragraph with <img src="inline.jpg" alt="Inline" /> in the middle.</p>
<img src="block.png" alt="Block" />
<p>End of content.</p>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain paragraph text
    assert!(result.contains("Start of paragraph"), "Should contain paragraph start");
    assert!(result.contains("in the middle"), "Should contain paragraph content");
    assert!(result.contains("End of content"), "Should contain paragraph end");

    // Should not contain either image
    assert!(!result.contains("![Inline]"), "Should not contain inline image");
    assert!(!result.contains("inline.jpg"), "Should not contain inline image URL");
    assert!(!result.contains("![Block]"), "Should not contain block image");
    assert!(!result.contains("block.png"), "Should not contain block image URL");
}

#[test]
fn test_skip_images_with_multiple_attributes() {
    // Test images with additional attributes (width, height, class, etc.)
    let html = r#"<img src="image.jpg" alt="Styled Image" width="500" height="300" class="responsive" data-lazy="true" />
<p>Image with attributes above.</p>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain text
    assert!(result.contains("Image with attributes"), "Should contain text");

    // Should not contain image or its attributes
    assert!(!result.contains("![Styled Image]"), "Should not contain image markdown");
    assert!(!result.contains("image.jpg"), "Should not contain image URL");
    assert!(!result.contains("500"), "Should not contain width attribute");
    assert!(!result.contains("300"), "Should not contain height attribute");
}

#[test]
fn test_skip_images_empty_document() {
    // Test that skip_images doesn't break on documents with only images
    let html = r#"<img src="image1.jpg" alt="Image 1" />
<img src="image2.png" alt="Image 2" />
<img src="image3.gif" alt="Image 3" />"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Result should be empty or near-empty (no meaningful content)
    // Just verify no images appear
    assert!(!result.contains("![Image 1]"), "Should not contain first image");
    assert!(!result.contains("![Image 2]"), "Should not contain second image");
    assert!(!result.contains("![Image 3]"), "Should not contain third image");
}

#[test]
fn test_skip_images_with_lists_and_images() {
    // Test that skip_images works correctly with lists containing images
    let html = r#"<ul>
<li>First item</li>
<li><img src="list-item.jpg" alt="List Item Image" /> Item with image</li>
<li>Third item</li>
</ul>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain list items
    assert!(result.contains("First item"), "Should contain first list item");
    assert!(result.contains("Item with image"), "Should contain list item text");
    assert!(result.contains("Third item"), "Should contain third list item");

    // Should not contain image
    assert!(
        !result.contains("![List Item Image]"),
        "Should not contain image in list"
    );
    assert!(
        !result.contains("list-item.jpg"),
        "Should not contain image URL in list"
    );
}

#[test]
fn test_skip_images_with_table_images() {
    // Test that skip_images works correctly with images inside tables
    let html = r#"<table>
<tr>
<td>Cell 1</td>
<td><img src="table-image.jpg" alt="Table Image" /></td>
</tr>
<tr>
<td>Cell 3</td>
<td>Cell 4</td>
</tr>
</table>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain table content
    assert!(result.contains("Cell 1"), "Should contain table cell 1");
    assert!(result.contains("Cell 3"), "Should contain table cell 3");
    assert!(result.contains("Cell 4"), "Should contain table cell 4");

    // Should not contain image
    assert!(!result.contains("![Table Image]"), "Should not contain image in table");
    assert!(
        !result.contains("table-image.jpg"),
        "Should not contain image URL in table"
    );
}

#[test]
fn test_skip_images_with_figure_figcaption() {
    // Test that skip_images removes images from figure elements
    let html = r#"<figure>
<img src="diagram.svg" alt="Diagram" />
<figcaption>This is a diagram caption</figcaption>
</figure>
<p>More content.</p>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain caption text
    assert!(
        result.contains("This is a diagram caption"),
        "Should contain figcaption text"
    );
    assert!(result.contains("More content"), "Should contain following content");

    // Should not contain image
    assert!(!result.contains("![Diagram]"), "Should not contain image markdown");
    assert!(!result.contains("diagram.svg"), "Should not contain image URL");
}

#[test]
fn test_skip_images_false_with_alt_text() {
    // When skip_images is false, verify images with alt text are converted correctly
    let html = r#"<p>Image below:</p>
<img src="image.jpg" alt="Test Alt Text" />
<p>Text below.</p>"#;

    let options = ConversionOptions {
        skip_images: false,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should include image with alt text
    assert!(
        result.contains("![Test Alt Text]"),
        "Should contain image markdown with alt text"
    );
    assert!(result.contains("image.jpg"), "Should contain image URL");
}

#[test]
fn test_skip_images_false_without_alt_text() {
    // When skip_images is false, verify images without alt text are handled
    let html = r#"<p>Image below:</p>
<img src="image.jpg" />
<p>Text below.</p>"#;

    let options = ConversionOptions {
        skip_images: false,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should include image (even without alt text, may use empty alt or URL)
    assert!(result.contains("image.jpg"), "Should contain image URL");
    // The exact format for images without alt text depends on implementation
}

#[test]
fn test_skip_images_with_picture_element() {
    // Test handling of picture elements with multiple sources
    let html = r#"<picture>
  <source srcset="image.webp" type="image/webp" />
  <source srcset="image.jpg" type="image/jpeg" />
  <img src="image.jpg" alt="Fallback Image" />
</picture>
<p>After picture element.</p>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should contain text after
    assert!(
        result.contains("After picture element"),
        "Should contain text after picture"
    );

    // Should not contain any images or sources
    assert!(
        !result.contains("![Fallback Image]"),
        "Should not contain fallback image"
    );
    assert!(!result.contains("image.webp"), "Should not contain webp source");
    assert!(!result.contains("image.jpg"), "Should not contain jpg source");
}

#[test]
fn test_skip_images_preserves_links_and_formatting() {
    // Verify that skip_images doesn't affect other markdown conversions
    let html = r#"<p>This is <strong>bold</strong>, <em>italic</em>, and <code>code</code>.</p>
<p>Here's a <a href="https://example.com">link</a>.</p>
<img src="ignored.jpg" alt="Ignored" />
<p>And a quote:</p>
<blockquote>
  <p>This is a blockquote.</p>
</blockquote>"#;

    let options = ConversionOptions {
        skip_images: true,
        ..Default::default()
    };

    let result = convert(html, Some(options)).unwrap();

    // Should preserve formatting
    assert!(result.contains("**bold**"), "Should preserve bold");
    assert!(result.contains("*italic*"), "Should preserve italic");
    assert!(result.contains("`code`"), "Should preserve code");

    // Should preserve links
    assert!(result.contains("[link]"), "Should preserve link text");
    assert!(result.contains("https://example.com"), "Should preserve link URL");

    // Should preserve blockquote
    assert!(result.contains("This is a blockquote"), "Should preserve blockquote");

    // Should not contain image
    assert!(!result.contains("![Ignored]"), "Should not contain image");
}
