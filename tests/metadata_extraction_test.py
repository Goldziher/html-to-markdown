from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Callable


def test_title_extraction(convert: Callable[..., str]) -> None:
    html = "<html><head><title>My Page Title</title></head><body><p>Content</p></body></html>"
    result = convert(html)
    expected = "---\ntitle: My Page Title\n---\n\nContent\n"
    assert result == expected


def test_meta_description(convert: Callable[..., str]) -> None:
    html = '<html><head><meta name="description" content="Page description"></head><body><p>Content</p></body></html>'
    result = convert(html)
    expected = "---\nmeta-description: Page description\n---\n\nContent\n"
    assert result == expected


def test_meta_keywords(convert: Callable[..., str]) -> None:
    html = '<html><head><meta name="keywords" content="keyword1, keyword2, keyword3"></head><body><p>Content</p></body></html>'
    result = convert(html)
    expected = "---\nmeta-keywords: keyword1, keyword2, keyword3\n---\n\nContent\n"
    assert result == expected


def test_meta_author(convert: Callable[..., str]) -> None:
    html = '<html><head><meta name="author" content="John Doe"></head><body><p>Content</p></body></html>'
    result = convert(html)
    expected = "---\nmeta-author: John Doe\n---\n\nContent\n"
    assert result == expected


def test_base_href(convert: Callable[..., str]) -> None:
    html = '<html><head><base href="https://example.com/"></head><body><p>Content</p></body></html>'
    result = convert(html)
    # URLs may be quoted in YAML
    assert result in (
        "---\nbase-href: https://example.com/\n---\n\nContent\n",
        '---\nbase-href: "https://example.com/"\n---\n\nContent\n',
    )


def test_canonical_link(convert: Callable[..., str]) -> None:
    html = '<html><head><link rel="canonical" href="https://example.com/page"></head><body><p>Content</p></body></html>'
    result = convert(html)
    # URLs may be quoted in YAML
    assert result in (
        "---\ncanonical: https://example.com/page\n---\n\nContent\n",
        '---\ncanonical: "https://example.com/page"\n---\n\nContent\n',
    )


def test_open_graph_metadata(convert: Callable[..., str]) -> None:
    html = """<html>
    <head>
        <meta property="og:title" content="OG Title">
        <meta property="og:description" content="OG Description">
        <meta property="og:image" content="https://example.com/image.jpg">
        <meta property="og:url" content="https://example.com/page">
    </head>
    <body><p>Content</p></body>
    </html>"""
    result = convert(html)
    assert "meta-og-title: OG Title" in result
    assert "meta-og-description: OG Description" in result
    # URLs may be quoted in YAML
    assert (
        "meta-og-image: https://example.com/image.jpg" in result
        or 'meta-og-image: "https://example.com/image.jpg"' in result
    )
    assert "meta-og-url: https://example.com/page" in result or 'meta-og-url: "https://example.com/page"' in result


def test_http_equiv_metadata(convert: Callable[..., str]) -> None:
    html = '<html><head><meta http-equiv="content-type" content="text/html; charset=UTF-8"></head><body><p>Content</p></body></html>'
    result = convert(html)
    expected = "---\nmeta-content-type: text/html; charset=UTF-8\n---\n\nContent\n"
    assert result == expected


def test_multiple_metadata(convert: Callable[..., str]) -> None:
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
    result = convert(html)
    assert "title: Page Title" in result
    assert "meta-description: Page description" in result
    assert "meta-author: John Doe" in result
    # URLs may be quoted in YAML
    assert "base-href: https://example.com/" in result or 'base-href: "https://example.com/"' in result
    assert "canonical: https://example.com/page" in result or 'canonical: "https://example.com/page"' in result


def test_metadata_with_special_characters(convert: Callable[..., str]) -> None:
    html = "<html><head><title>Title with --> comment closer</title></head><body><p>Content</p></body></html>"
    result = convert(html)
    # In YAML frontmatter, we need to quote strings with special characters
    assert "title:" in result
    assert "---" in result
    assert "Content\n" in result


def test_empty_metadata_values(convert: Callable[..., str]) -> None:
    html = '<html><head><meta name="description" content=""></head><body><p>Content</p></body></html>'
    result = convert(html)
    assert result in ("---\nmeta-description: ''\n---\n\nContent\n", "---\nmeta-description: \n---\n\nContent\n")


def test_no_metadata(convert: Callable[..., str]) -> None:
    html = "<p>Content</p>"
    result = convert(html)
    assert result == "Content\n"


def test_extract_metadata_false(convert: Callable[..., str]) -> None:
    html = "<html><head><title>My Title</title></head><body><p>Content</p></body></html>"
    result = convert(html, extract_metadata=False)
    assert result == "Content\n"
    assert "---" not in result
    assert "title:" not in result


def test_metadata_in_inline_mode(convert: Callable[..., str]) -> None:
    html = "<html><head><title>My Title</title></head><body><p>Content</p></body></html>"
    result = convert(html, convert_as_inline=True)
    assert result == "Content\n"
    assert "---" not in result


def test_link_relations(convert: Callable[..., str]) -> None:
    html = """<html>
    <head>
        <link rel="author" href="https://example.com/author">
        <link rel="license" href="https://example.com/license">
        <link rel="alternate" href="https://example.com/alternate">
    </head>
    <body><p>Content</p></body>
    </html>"""
    result = convert(html)
    # URLs may be quoted in YAML
    assert "link-author: https://example.com/author" in result or 'link-author: "https://example.com/author"' in result
    assert (
        "link-license: https://example.com/license" in result or 'link-license: "https://example.com/license"' in result
    )
    assert (
        "link-alternate: https://example.com/alternate" in result
        or 'link-alternate: "https://example.com/alternate"' in result
    )


def test_sorted_metadata_output(convert: Callable[..., str]) -> None:
    html = """<html>
    <head>
        <title>Title</title>
        <meta name="author" content="Author">
        <meta name="description" content="Description">
        <base href="https://example.com/">
    </head>
    <body><p>Content</p></body>
    </html>"""
    result = convert(html)
    # YAML frontmatter ends with ---
    metadata_end = result.index("---", 3) + 3  # Find second occurrence of ---
    metadata_block = result[4 : metadata_end - 3]  # Skip first --- and last ---
    lines = [line.strip() for line in metadata_block.split("\n") if line.strip()]
    keys = [line.split(":")[0] for line in lines if ":" in line]
    assert keys == sorted(keys)


def test_whitespace_in_title(convert: Callable[..., str]) -> None:
    html = "<html><head><title>  Title with   spaces  </title></head><body><p>Content</p></body></html>"
    result = convert(html)
    expected = "---\ntitle: Title with spaces\n---\n\nContent\n"
    assert result == expected
