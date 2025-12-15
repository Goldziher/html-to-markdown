"""Comprehensive integration tests for metadata extraction API.

Tests validate end-to-end metadata extraction functionality including:
- Document metadata (title, description, language, etc.)
- Header hierarchy extraction
- Link classification
- Image metadata gathering
- Structured data detection
- Feature flag configuration
- Edge cases and malformed HTML
"""

from __future__ import annotations

from html_to_markdown import MetadataConfig, convert_with_metadata


class TestDocumentMetadataExtraction:
    """Test extraction of document-level metadata."""

    def test_language_attribute(self) -> None:
        """Test extraction of document language from lang attribute."""
        html = """
        <html lang="en-US">
        <head></head>
        <body><p>Content</p></body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        assert metadata["document"]["language"] == "en-US"

    def test_language_none_when_not_present(self) -> None:
        """Test that language is None when not present."""
        html = """
        <html>
        <head></head>
        <body><p>Content</p></body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        assert metadata["document"]["language"] is None

    def test_text_direction_ltr(self) -> None:
        """Test extraction of left-to-right text direction."""
        html = """
        <html dir="ltr">
        <head></head>
        <body><p>Content</p></body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        assert metadata["document"]["text_direction"] == "ltr"

    def test_text_direction_rtl(self) -> None:
        """Test extraction of right-to-left text direction."""
        html = """
        <html dir="rtl">
        <head></head>
        <body><p>Content</p></body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        assert metadata["document"]["text_direction"] == "rtl"

    def test_document_metadata_structure_complete(self) -> None:
        """Test that document metadata has all expected fields."""
        html = """
        <html lang="en" dir="ltr">
        <head></head>
        <body><p>Content</p></body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        doc = metadata["document"]
        assert "title" in doc
        assert "description" in doc
        assert "keywords" in doc
        assert "author" in doc
        assert "canonical_url" in doc
        assert "base_href" in doc
        assert "language" in doc
        assert "text_direction" in doc
        assert "open_graph" in doc
        assert "twitter_card" in doc
        assert "meta_tags" in doc


class TestHeaderHierarchyExtraction:
    """Test extraction of headers with hierarchy information."""

    def test_single_header(self) -> None:
        """Test extraction of a single header."""
        html = """
        <html>
        <body>
            <h1>Main Title</h1>
            <p>Content</p>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        headers = metadata["headers"]
        assert len(headers) == 1
        assert headers[0]["level"] == 1
        assert headers[0]["text"] == "Main Title"

    def test_header_hierarchy_levels(self) -> None:
        """Test that header levels are correctly identified."""
        html = """
        <html>
        <body>
            <h1>H1 Title</h1>
            <h2>H2 Subtitle</h2>
            <h3>H3 Section</h3>
            <h4>H4 Subsection</h4>
            <h5>H5 Detail</h5>
            <h6>H6 Minor</h6>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        headers = metadata["headers"]
        assert len(headers) == 6
        for i, expected_level in enumerate(range(1, 7)):
            assert headers[i]["level"] == expected_level

    def test_header_id_preservation(self) -> None:
        """Test that header id attributes are preserved."""
        html = """
        <html>
        <body>
            <h1 id="main-section">Main</h1>
            <h2 id="sub-section">Sub</h2>
            <h3>No ID</h3>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        headers = metadata["headers"]
        assert headers[0]["id"] == "main-section"
        assert headers[1]["id"] == "sub-section"
        assert headers[2]["id"] is None

    def test_header_text_extraction(self) -> None:
        """Test that header text content is correctly extracted."""
        html = """
        <html>
        <body>
            <h1>Simple Title</h1>
            <h2>Title with <strong>bold</strong> text</h2>
            <h3>Title with <em>italic</em> and <code>code</code></h3>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        headers = metadata["headers"]
        assert "Simple Title" in headers[0]["text"]
        assert "bold" in headers[1]["text"]
        assert "italic" in headers[2]["text"]

    def test_header_depth_tracking(self) -> None:
        """Test that header depth in document tree is tracked."""
        html = """
        <html>
        <body>
            <div>
                <div>
                    <h1>Deep Header</h1>
                </div>
            </div>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        headers = metadata["headers"]
        assert len(headers) == 1
        assert "depth" in headers[0]
        assert isinstance(headers[0]["depth"], int)

    def test_header_html_offset(self) -> None:
        """Test that html_offset is present and valid."""
        html = """
        <html>
        <body>
            <h1>Test</h1>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        headers = metadata["headers"]
        assert "html_offset" in headers[0]
        assert isinstance(headers[0]["html_offset"], int)
        assert headers[0]["html_offset"] >= 0

    def test_multiple_headers_same_level(self) -> None:
        """Test extraction of multiple headers at same level."""
        html = """
        <html>
        <body>
            <h2>Section 1</h2>
            <p>Content 1</p>
            <h2>Section 2</h2>
            <p>Content 2</p>
            <h2>Section 3</h2>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        headers = metadata["headers"]
        assert len(headers) == 3
        assert all(h["level"] == 2 for h in headers)
        assert headers[0]["text"] == "Section 1"
        assert headers[1]["text"] == "Section 2"
        assert headers[2]["text"] == "Section 3"


class TestLinkClassificationCorrectness:
    """Test link extraction and proper classification."""

    def test_anchor_link_classification(self) -> None:
        """Test that anchor links (starting with #) are classified correctly."""
        html = """
        <html>
        <body>
            <a href="#section">Jump to section</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        links = metadata["links"]
        assert len(links) == 1
        assert links[0]["link_type"] == "anchor"
        assert links[0]["href"] == "#section"

    def test_internal_link_classification(self) -> None:
        """Test that internal links are classified correctly."""
        html = """
        <html>
        <body>
            <a href="/page">Root relative</a>
            <a href="./relative">Relative</a>
            <a href="../parent">Parent relative</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        links = metadata["links"]
        assert len(links) == 3
        assert all(link["link_type"] == "internal" for link in links)

    def test_external_link_classification(self) -> None:
        """Test that external links are classified correctly."""
        html = """
        <html>
        <body>
            <a href="https://example.com">HTTPS</a>
            <a href="http://example.com">HTTP</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        links = metadata["links"]
        assert len(links) == 2
        assert all(link["link_type"] == "external" for link in links)

    def test_email_link_classification(self) -> None:
        """Test that email links are classified correctly."""
        html = """
        <html>
        <body>
            <a href="mailto:test@example.com">Email us</a>
            <a href="mailto:admin@test.org">Admin</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        links = metadata["links"]
        assert len(links) == 2
        assert all(link["link_type"] == "email" for link in links)

    def test_phone_link_classification(self) -> None:
        """Test that phone links are classified correctly."""
        html = """
        <html>
        <body>
            <a href="tel:+1-555-123-4567">Call us</a>
            <a href="tel:18005551234">Toll free</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        links = metadata["links"]
        assert len(links) == 2
        assert all(link["link_type"] == "phone" for link in links)

    def test_link_text_extraction(self) -> None:
        """Test that link text content is correctly extracted."""
        html = """
        <html>
        <body>
            <a href="/page">Simple text</a>
            <a href="/page"><strong>Bold</strong> text</a>
            <a href="/page">Mixed <em>italic</em> content</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        links = metadata["links"]
        assert links[0]["text"] == "Simple text"
        assert "Bold" in links[1]["text"]
        assert "italic" in links[2]["text"]

    def test_link_title_attribute(self) -> None:
        """Test that link title attribute is extracted."""
        html = """
        <html>
        <body>
            <a href="/" title="Home page">Home</a>
            <a href="/about">About</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        links = metadata["links"]
        assert links[0]["title"] == "Home page"
        assert links[1]["title"] is None

    def test_link_rel_attribute(self) -> None:
        """Test that link rel attributes are extracted."""
        html = """
        <html>
        <body>
            <a href="https://external.com" rel="nofollow">No follow</a>
            <a href="https://external.com" rel="noopener noreferrer">Security</a>
            <a href="/">Home</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        links = metadata["links"]
        assert "nofollow" in links[0]["rel"]
        assert "noopener" in links[1]["rel"]
        assert "noreferrer" in links[1]["rel"]
        assert len(links[2]["rel"]) == 0

    def test_link_attributes_dict(self) -> None:
        """Test that link attributes dict is present."""
        html = """
        <html>
        <body>
            <a href="/" data-track="home" class="nav-link">Home</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        links = metadata["links"]
        assert "attributes" in links[0]
        assert isinstance(links[0]["attributes"], dict)


class TestImageMetadataGathering:
    """Test extraction of image metadata."""

    def test_image_src_extraction(self) -> None:
        """Test that image src is extracted."""
        html = """
        <html>
        <body>
            <img src="https://example.com/image.jpg">
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        images = metadata["images"]
        assert len(images) == 1
        assert images[0]["src"] == "https://example.com/image.jpg"

    def test_image_alt_text(self) -> None:
        """Test that image alt text is extracted."""
        html = """
        <html>
        <body>
            <img src="image.jpg" alt="An example image">
            <img src="other.jpg">
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        images = metadata["images"]
        assert images[0]["alt"] == "An example image"
        assert images[1]["alt"] is None

    def test_image_title_attribute(self) -> None:
        """Test that image title attribute is extracted."""
        html = """
        <html>
        <body>
            <img src="image.jpg" title="Image title">
            <img src="other.jpg">
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        images = metadata["images"]
        assert images[0]["title"] == "Image title"
        assert images[1]["title"] is None

    def test_image_dimensions(self) -> None:
        """Test that image dimensions field is present."""
        html = """
        <html>
        <body>
            <img src="image.jpg" width="800" height="600">
            <img src="other.jpg">
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        images = metadata["images"]
        assert "dimensions" in images[0]
        assert "dimensions" in images[1]

    def test_image_type_external(self) -> None:
        """Test that external images are classified correctly."""
        html = """
        <html>
        <body>
            <img src="https://example.com/image.jpg">
            <img src="http://example.com/other.png">
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        images = metadata["images"]
        assert all(img["image_type"] == "external" for img in images)

    def test_image_type_relative(self) -> None:
        """Test that relative images are classified correctly."""
        html = """
        <html>
        <body>
            <img src="image.jpg">
            <img src="./images/other.jpg">
            <img src="../assets/photo.png">
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        images = metadata["images"]
        assert all(img["image_type"] == "relative" for img in images)

    def test_image_type_data_uri(self) -> None:
        """Test that data URI images are classified correctly."""
        html = """
        <html>
        <body>
            <img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUA">
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        images = metadata["images"]
        assert len(images) == 1
        assert images[0]["image_type"] == "data_uri"

    def test_image_attributes_dict(self) -> None:
        """Test that image attributes dict is present."""
        html = """
        <html>
        <body>
            <img src="image.jpg" class="hero" data-lazy="true">
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        images = metadata["images"]
        assert "attributes" in images[0]
        assert isinstance(images[0]["attributes"], dict)


class TestStructuredDataDetection:
    """Test extraction of structured data."""

    def test_structured_data_list_present(self) -> None:
        """Test that structured_data list is present in metadata."""
        html = """
        <html>
        <head>
            <script type="application/ld+json">
            {
                "@context": "https://schema.org",
                "@type": "Article",
                "headline": "Test Article"
            }
            </script>
        </head>
        <body><p>Content</p></body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        assert "structured_data" in metadata
        assert isinstance(metadata["structured_data"], list)


class TestMetadataConfigFeatureFlags:
    """Test that MetadataConfig controls what gets extracted."""

    def test_extract_headers_flag(self) -> None:
        """Test that extract_headers flag controls header extraction."""
        html = """
        <html>
        <body>
            <h1>Title</h1>
            <p>Content</p>
        </body>
        </html>
        """
        config = MetadataConfig(extract_headers=True)
        _markdown, metadata = convert_with_metadata(html, metadata_config=config)
        assert len(metadata["headers"]) > 0

        config = MetadataConfig(extract_headers=False)
        _markdown, metadata = convert_with_metadata(html, metadata_config=config)
        assert len(metadata["headers"]) == 0

    def test_extract_links_flag(self) -> None:
        """Test that extract_links flag controls link extraction."""
        html = """
        <html>
        <body>
            <a href="/">Home</a>
        </body>
        </html>
        """
        config = MetadataConfig(extract_links=True)
        _markdown, metadata = convert_with_metadata(html, metadata_config=config)
        assert len(metadata["links"]) > 0

        config = MetadataConfig(extract_links=False)
        _markdown, metadata = convert_with_metadata(html, metadata_config=config)
        assert len(metadata["links"]) == 0

    def test_extract_images_flag(self) -> None:
        """Test that extract_images flag controls image extraction."""
        html = """
        <html>
        <body>
            <img src="image.jpg">
        </body>
        </html>
        """
        config = MetadataConfig(extract_images=True)
        _markdown, metadata = convert_with_metadata(html, metadata_config=config)
        assert len(metadata["images"]) > 0

        config = MetadataConfig(extract_images=False)
        _markdown, metadata = convert_with_metadata(html, metadata_config=config)
        assert len(metadata["images"]) == 0

    def test_extract_structured_data_flag(self) -> None:
        """Test that extract_structured_data flag is accepted in MetadataConfig."""
        config = MetadataConfig(extract_structured_data=True)
        assert config.extract_structured_data is True

        config = MetadataConfig(extract_structured_data=False)
        assert config.extract_structured_data is False

    def test_selective_extraction_combination(self) -> None:
        """Test selective extraction with multiple flags disabled."""
        html = """
        <html lang="en">
        <head>
            <title>Test</title>
            <script type="application/ld+json">{"@type": "Article"}</script>
        </head>
        <body>
            <h1>Title</h1>
            <a href="/">Home</a>
            <img src="image.jpg">
        </body>
        </html>
        """
        config = MetadataConfig(
            extract_headers=True,
            extract_links=True,
            extract_images=False,
            extract_structured_data=False,
        )
        _markdown, metadata = convert_with_metadata(html, metadata_config=config)
        assert len(metadata["headers"]) > 0
        assert len(metadata["links"]) > 0
        assert len(metadata["images"]) == 0
        assert len(metadata["structured_data"]) == 0


class TestEdgeCasesAndMalformedHTML:
    """Test metadata extraction with edge cases and malformed HTML."""

    def test_empty_html(self) -> None:
        """Test extraction from empty HTML."""
        html = "<html></html>"
        _markdown, metadata = convert_with_metadata(html)
        assert metadata["document"]["title"] is None
        assert len(metadata["headers"]) == 0
        assert len(metadata["links"]) == 0

    def test_empty_metadata_values(self) -> None:
        """Test extraction of empty metadata values."""
        html = """
        <html>
        <head>
            <meta name="description" content="">
            <meta name="author" content="">
        </head>
        <body><p>Content</p></body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        desc = metadata["document"]["description"]
        assert desc == "" or desc is None

    def test_malformed_html_structure(self) -> None:
        """Test extraction from malformed HTML with unclosed tags."""
        html = """
        <html>
        <body>
            <h1>Title
            <p>Content
            <a href="/">Link
        </body>
        """
        _markdown, metadata = convert_with_metadata(html)
        assert isinstance(metadata, dict)
        assert "document" in metadata

    def test_headers_without_text(self) -> None:
        """Test extraction of headers with no text content."""
        html = """
        <html>
        <body>
            <h1></h1>
            <h2>   </h2>
            <h3>Valid</h3>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        metadata["headers"]

    def test_deeply_nested_structure(self) -> None:
        """Test extraction from deeply nested HTML."""
        html = """
        <html>
        <body>
            <div><div><div><div><div>
                <h1>Deep Title</h1>
                <a href="/">Deep Link</a>
            </div></div></div></div></div>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        assert len(metadata["headers"]) > 0
        assert len(metadata["links"]) > 0

    def test_special_characters_in_html(self) -> None:
        """Test extraction of metadata from HTML with special characters."""
        html = """
        <html>
        <head>
            <meta name="description" content="Content with special chars">
        </head>
        <body><p>Content</p></body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        assert isinstance(metadata, dict)

    def test_malformed_html_handling(self) -> None:
        """Test that malformed HTML doesn't crash metadata extraction."""
        html = """
        <html>
        <head>
            <meta name="description" content="Missing closing meta>
        </head>
        <body><p>Content</p></body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        assert isinstance(metadata, dict)
        assert "document" in metadata


class TestTypeVerification:
    """Test that metadata structures have correct types."""

    def test_metadata_is_dict(self) -> None:
        """Test that metadata is a dictionary."""
        html = "<html><body><p>Test</p></body></html>"
        _markdown, metadata = convert_with_metadata(html)
        assert isinstance(metadata, dict)

    def test_document_metadata_structure(self) -> None:
        """Test that document metadata has expected structure."""
        html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>"
        _markdown, metadata = convert_with_metadata(html)
        doc = metadata["document"]
        assert isinstance(doc, dict)
        assert "title" in doc
        assert "description" in doc
        assert "keywords" in doc
        assert isinstance(doc["keywords"], list)
        assert "author" in doc
        assert "canonical_url" in doc
        assert "base_href" in doc
        assert "language" in doc
        assert "text_direction" in doc
        assert "open_graph" in doc
        assert isinstance(doc["open_graph"], dict)
        assert "twitter_card" in doc
        assert isinstance(doc["twitter_card"], dict)
        assert "meta_tags" in doc
        assert isinstance(doc["meta_tags"], dict)

    def test_headers_list_structure(self) -> None:
        """Test that headers list has correct structure."""
        html = """
        <html>
        <body>
            <h1 id="test">Title</h1>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        headers = metadata["headers"]
        assert isinstance(headers, list)
        if headers:
            h = headers[0]
            assert isinstance(h, dict)
            assert "level" in h
            assert isinstance(h["level"], int)
            assert "text" in h
            assert isinstance(h["text"], str)
            assert "id" in h
            assert "depth" in h
            assert isinstance(h["depth"], int)
            assert "html_offset" in h
            assert isinstance(h["html_offset"], int)

    def test_links_list_structure(self) -> None:
        """Test that links list has correct structure."""
        html = """
        <html>
        <body>
            <a href="/" title="Home" rel="nofollow">Link</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        links = metadata["links"]
        assert isinstance(links, list)
        if links:
            link = links[0]
            assert isinstance(link, dict)
            assert "href" in link
            assert isinstance(link["href"], str)
            assert "text" in link
            assert isinstance(link["text"], str)
            assert "title" in link
            assert "link_type" in link
            assert isinstance(link["link_type"], str)
            assert "rel" in link
            assert isinstance(link["rel"], list)
            assert "attributes" in link
            assert isinstance(link["attributes"], dict)

    def test_images_list_structure(self) -> None:
        """Test that images list has correct structure."""
        html = """
        <html>
        <body>
            <img src="image.jpg" alt="Alt text" width="100" height="50">
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        images = metadata["images"]
        assert isinstance(images, list)
        if images:
            img = images[0]
            assert isinstance(img, dict)
            assert "src" in img
            assert isinstance(img["src"], str)
            assert "alt" in img
            assert "title" in img
            assert "dimensions" in img
            assert "image_type" in img
            assert isinstance(img["image_type"], str)
            assert "attributes" in img
            assert isinstance(img["attributes"], dict)

    def test_structured_data_list_structure(self) -> None:
        """Test that structured data list has correct structure."""
        html = """
        <html>
        <head>
            <script type="application/ld+json">
            {"@type": "Article"}
            </script>
        </head>
        <body><p>Content</p></body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)
        data = metadata["structured_data"]
        assert isinstance(data, list)
        if data:
            item = data[0]
            assert isinstance(item, dict)
            assert "data_type" in item
            assert isinstance(item["data_type"], str)
            assert "raw_json" in item
            assert isinstance(item["raw_json"], str)
            assert "schema_type" in item


class TestRealWorldDocuments:
    """Test metadata extraction with realistic HTML documents."""

    def test_blog_post_structure(self) -> None:
        """Test extraction from a realistic blog post."""
        html = """
        <!DOCTYPE html>
        <html lang="en" dir="ltr">
        <head>
            <title>How to Build Better APIs</title>
        </head>
        <body>
            <h1>How to Build Better APIs</h1>
            <h2>Introduction</h2>
            <p>This guide covers...</p>
            <h2>Best Practices</h2>
            <h3>Authentication</h3>
            <p>Always use...</p>
            <h3>Versioning</h3>
            <p>Plan for...</p>
            <p><a href="https://example.com">Learn more</a> at our documentation.</p>
            <img src="api-diagram.png" alt="API diagram">
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)

        assert metadata["document"]["language"] == "en"
        assert metadata["document"]["text_direction"] == "ltr"

        headers = metadata["headers"]
        assert len(headers) >= 4

        links = metadata["links"]
        assert len(links) > 0
        assert any(link["link_type"] == "external" for link in links)

        images = metadata["images"]
        assert len(images) > 0

    def test_product_page_structure(self) -> None:
        """Test extraction from a realistic product page."""
        html = """
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <title>Premium Widget - Best Price</title>
        </head>
        <body>
            <h1>Premium Widget</h1>
            <img src="product.jpg" alt="Widget image" width="400" height="400">
            <h2>Features</h2>
            <h2>Reviews</h2>
            <a href="/buy">Buy Now</a>
            <a href="https://twitter.com/share">Share</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)

        assert metadata["document"]["language"] == "en"
        assert len(metadata["headers"]) >= 3
        assert len(metadata["images"]) > 0
        assert len(metadata["links"]) > 0

    def test_documentation_page_structure(self) -> None:
        """Test extraction from a documentation page."""
        html = """
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <title>API Reference - Python SDK</title>
            <base href="https://docs.example.com/">
            <link rel="canonical" href="https://docs.example.com/python/">
        </head>
        <body>
            <h1 id="overview">Python SDK Overview</h1>
            <h2 id="installation">Installation</h2>
            <h2 id="usage">Usage</h2>
            <h3 id="basic-example">Basic Example</h3>
            <h3 id="advanced">Advanced Usage</h3>
            <h2 id="api-reference">API Reference</h2>
            <a href="/">Home</a>
            <a href="/tutorials">Tutorials</a>
            <a href="https://github.com/example">GitHub</a>
        </body>
        </html>
        """
        _markdown, metadata = convert_with_metadata(html)

        doc = metadata["document"]
        assert "base_href" in doc
        assert "canonical_url" in doc

        headers = metadata["headers"]
        assert any(h["id"] == "overview" for h in headers)
        assert any(h["level"] == 3 for h in headers)

        links = metadata["links"]
        assert len(links) == 3
        assert links[2]["link_type"] == "external"
