from html_to_markdown import convert_to_markdown


def test_title_extraction() -> None:
    html = "<html><head><title>My Page Title</title></head><body><p>Content</p></body></html>"
    result = convert_to_markdown(html)
    expected = "<!--\ntitle: My Page Title\n-->\n\nContent\n\n"
    assert result == expected


def test_meta_description() -> None:
    html = '<html><head><meta name="description" content="Page description"></head><body><p>Content</p></body></html>'
    result = convert_to_markdown(html)
    expected = "<!--\nmeta-description: Page description\n-->\n\nContent\n\n"
    assert result == expected


def test_meta_keywords() -> None:
    html = '<html><head><meta name="keywords" content="keyword1, keyword2, keyword3"></head><body><p>Content</p></body></html>'
    result = convert_to_markdown(html)
    expected = "<!--\nmeta-keywords: keyword1, keyword2, keyword3\n-->\n\nContent\n\n"
    assert result == expected


def test_meta_author() -> None:
    html = '<html><head><meta name="author" content="John Doe"></head><body><p>Content</p></body></html>'
    result = convert_to_markdown(html)
    expected = "<!--\nmeta-author: John Doe\n-->\n\nContent\n\n"
    assert result == expected


def test_base_href() -> None:
    html = '<html><head><base href="https://example.com/"></head><body><p>Content</p></body></html>'
    result = convert_to_markdown(html)
    expected = "<!--\nbase-href: https://example.com/\n-->\n\nContent\n\n"
    assert result == expected


def test_canonical_link() -> None:
    html = '<html><head><link rel="canonical" href="https://example.com/page"></head><body><p>Content</p></body></html>'
    result = convert_to_markdown(html)
    expected = "<!--\ncanonical: https://example.com/page\n-->\n\nContent\n\n"
    assert result == expected


def test_open_graph_metadata() -> None:
    html = """<html>
    <head>
        <meta property="og:title" content="OG Title">
        <meta property="og:description" content="OG Description">
        <meta property="og:image" content="https://example.com/image.jpg">
        <meta property="og:url" content="https://example.com/page">
    </head>
    <body><p>Content</p></body>
    </html>"""
    result = convert_to_markdown(html)
    assert "meta-og-title: OG Title" in result
    assert "meta-og-description: OG Description" in result
    assert "meta-og-image: https://example.com/image.jpg" in result
    assert "meta-og-url: https://example.com/page" in result


def test_http_equiv_metadata() -> None:
    html = '<html><head><meta http-equiv="content-type" content="text/html; charset=UTF-8"></head><body><p>Content</p></body></html>'
    result = convert_to_markdown(html)
    expected = "<!--\nmeta-content-type: text/html; charset=UTF-8\n-->\n\nContent\n\n"
    assert result == expected


def test_multiple_metadata() -> None:
    html = """<html>
    <head>
        <title>Page Title</title>
        <meta name="description" content="Page description">
        <meta name="author" content="John Doe">
        <base href="https://example.com/">
        <link rel="canonical" href="https://example.com/page">
    </head>
    <body><p>Content</p></body>
    </html>"""
    result = convert_to_markdown(html)
    assert "title: Page Title" in result
    assert "meta-description: Page description" in result
    assert "meta-author: John Doe" in result
    assert "base-href: https://example.com/" in result
    assert "canonical: https://example.com/page" in result


def test_metadata_with_special_characters() -> None:
    html = "<html><head><title>Title with --> comment closer</title></head><body><p>Content</p></body></html>"
    result = convert_to_markdown(html)
    expected = "<!--\ntitle: Title with --&gt; comment closer\n-->\n\nContent\n\n"
    assert result == expected


def test_empty_metadata_values() -> None:
    html = '<html><head><meta name="description" content=""></head><body><p>Content</p></body></html>'
    result = convert_to_markdown(html)

    expected = "<!--\nmeta-description: \n-->\n\nContent\n\n"
    assert result == expected


def test_no_metadata() -> None:
    html = "<p>Content</p>"
    result = convert_to_markdown(html)

    assert result == "Content\n\n"


def test_extract_metadata_false() -> None:
    html = "<html><head><title>My Title</title></head><body><p>Content</p></body></html>"
    result = convert_to_markdown(html, extract_metadata=False)

    assert result == "Content\n\n"
    assert "<!--" not in result


def test_metadata_in_inline_mode() -> None:
    html = "<html><head><title>My Title</title></head><body><p>Content</p></body></html>"
    result = convert_to_markdown(html, convert_as_inline=True)

    assert result == "Content"
    assert "<!--" not in result


def test_link_relations() -> None:
    html = """<html>
    <head>
        <link rel="author" href="https://example.com/author">
        <link rel="license" href="https://example.com/license">
        <link rel="alternate" href="https://example.com/alternate">
    </head>
    <body><p>Content</p></body>
    </html>"""
    result = convert_to_markdown(html)
    assert "link-author: https://example.com/author" in result
    assert "link-license: https://example.com/license" in result
    assert "link-alternate: https://example.com/alternate" in result


def test_sorted_metadata_output() -> None:
    html = """<html>
    <head>
        <title>Title</title>
        <meta name="author" content="Author">
        <meta name="description" content="Description">
        <base href="https://example.com/">
    </head>
    <body><p>Content</p></body>
    </html>"""
    result = convert_to_markdown(html)

    metadata_end = result.index("-->") + 3
    metadata_block = result[:metadata_end]

    lines = metadata_block.split("\n")[1:-1]
    keys = [line.split(":")[0] for line in lines if line]
    assert keys == sorted(keys)


def test_whitespace_in_title() -> None:
    html = "<html><head><title>  Title with   spaces  </title></head><body><p>Content</p></body></html>"
    result = convert_to_markdown(html)
    expected = "<!--\ntitle: Title with spaces\n-->\n\nContent\n\n"
    assert result == expected
