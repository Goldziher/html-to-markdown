from __future__ import annotations

from html_to_markdown import convert_to_markdown


def test_article_element() -> None:
    """Test conversion of HTML article element."""
    html = """
    <article>
        <header><h1>Article Title</h1></header>
        <section><p>Article content here.</p></section>
    </article>
    """
    expected = """<!-- article -->
<!-- header -->
Article Title
=============
<!-- /header -->

<!-- section -->
Article content here.
<!-- /section -->
<!-- /article -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_section_element() -> None:
    """Test conversion of HTML section element."""
    html = """
    <section>
        <h2>Section Title</h2>
        <p>Section content.</p>
    </section>
    """
    expected = """<!-- section -->
Section Title
-------------

Section content.
<!-- /section -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_nav_element() -> None:
    """Test conversion of HTML nav element."""
    html = """
    <nav>
        <ul>
            <li><a href="/home">Home</a></li>
            <li><a href="/about">About</a></li>
        </ul>
    </nav>
    """
    expected = """<!-- nav -->
* [Home](/home)
* [About](/about)
<!-- /nav -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_aside_element() -> None:
    """Test conversion of HTML aside element."""
    html = """
    <aside>
        <h3>Sidebar</h3>
        <p>Additional information.</p>
    </aside>
    """
    expected = """<!-- aside -->
### Sidebar

Additional information.
<!-- /aside -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_header_element() -> None:
    """Test conversion of HTML header element."""
    html = """
    <header>
        <h1>Page Title</h1>
        <p>Page description</p>
    </header>
    """
    expected = """<!-- header -->
Page Title
==========

Page description
<!-- /header -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_footer_element() -> None:
    """Test conversion of HTML footer element."""
    html = """
    <footer>
        <p>Copyright 2024</p>
        <p>Contact: info@example.com</p>
    </footer>
    """
    expected = """<!-- footer -->
Copyright 2024

Contact: info@example.com
<!-- /footer -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_main_element() -> None:
    """Test conversion of HTML main element."""
    html = """
    <main>
        <h1>Main Content</h1>
        <p>This is the main content area.</p>
    </main>
    """
    expected = """<!-- main -->
Main Content
============

This is the main content area.
<!-- /main -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_semantic_elements_as_inline() -> None:
    """Test semantic elements when converted as inline content."""
    html = '<p>This is an <article>inline article</article> element.</p>'
    expected = "This is an inline article element.\n\n"
    result = convert_to_markdown(html, convert_as_inline=True)
    assert result == expected


def test_nested_semantic_elements() -> None:
    """Test nested semantic elements."""
    html = """
    <article>
        <header>
            <h1>Article Title</h1>
        </header>
        <main>
            <section>
                <h2>Section Title</h2>
                <p>Content in section.</p>
            </section>
        </main>
        <aside>
            <p>Side note.</p>
        </aside>
    </article>
    """
    expected = """<!-- article -->
<!-- header -->
Article Title
=============
<!-- /header -->

<!-- main -->
<!-- section -->
Section Title
-------------

Content in section.
<!-- /section -->
<!-- /main -->

<!-- aside -->
Side note.
<!-- /aside -->

<!-- /article -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_empty_semantic_elements() -> None:
    """Test empty semantic elements."""
    html = "<article></article>"
    expected = ""
    result = convert_to_markdown(html)
    assert result.strip() == expected


def test_semantic_elements_with_whitespace_only() -> None:
    """Test semantic elements with only whitespace content."""
    html = "<section>   \n\t  </section>"
    expected = ""
    result = convert_to_markdown(html)
    assert result.strip() == expected


def test_semantic_elements_with_complex_content() -> None:
    """Test semantic elements with complex nested content."""
    html = """
    <article>
        <header>
            <h1>Complex Article</h1>
            <nav>
                <a href="#toc">Table of Contents</a>
            </nav>
        </header>
        <section>
            <h2>Introduction</h2>
            <p>This is the <strong>introduction</strong>.</p>
            <blockquote>
                <p>An important quote.</p>
            </blockquote>
        </section>
        <aside>
            <h3>Related Links</h3>
            <ul>
                <li><a href="/link1">Link 1</a></li>
                <li><a href="/link2">Link 2</a></li>
            </ul>
        </aside>
        <footer>
            <p>Published on <time>2024-01-01</time></p>
        </footer>
    </article>
    """
    expected = """<!-- article -->
<!-- header -->
Complex Article
===============
<!-- /header -->

<!-- nav -->
[Table of Contents](#toc)
<!-- /nav -->

<!-- section -->
Introduction
------------

This is the **introduction**.

> An important quote.

<!-- /section -->

<!-- aside -->
### Related Links

* [Link 1](/link1)
* [Link 2](/link2)
<!-- /aside -->

<!-- footer -->
Published on 2024-01-01
<!-- /footer -->

<!-- /article -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_semantic_elements_with_tables() -> None:
    """Test semantic elements containing tables."""
    html = """
    <section>
        <h2>Data Section</h2>
        <table>
            <tr>
                <th>Name</th>
                <th>Value</th>
            </tr>
            <tr>
                <td>Item 1</td>
                <td>100</td>
            </tr>
        </table>
    </section>
    """
    expected = """<!-- section -->
Data Section
------------

| Name | Value |
| --- | --- |
| Item 1 | 100 |

<!-- /section -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_semantic_elements_with_lists() -> None:
    """Test semantic elements containing lists."""
    html = """
    <nav>
        <ul>
            <li>Home</li>
            <li>About</li>
            <li>Contact</li>
        </ul>
    </nav>
    """
    expected = """<!-- nav -->
* Home
* About
* Contact
<!-- /nav -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_semantic_elements_with_code_blocks() -> None:
    """Test semantic elements containing code blocks."""
    html = """
    <section>
        <h2>Code Example</h2>
        <pre><code>print("Hello, World!")</code></pre>
    </section>
    """
    expected = """<!-- section -->
Code Example
------------

```print("Hello, World!")```
<!-- /section -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_multiple_semantic_elements() -> None:
    """Test multiple semantic elements in sequence."""
    html = """
    <header>
        <h1>Page Header</h1>
    </header>
    <main>
        <p>Main content</p>
    </main>
    <aside>
        <p>Sidebar</p>
    </aside>
    <footer>
        <p>Page footer</p>
    </footer>
    """
    expected = """<!-- header -->
Page Header
===========
<!-- /header -->

<!-- main -->
Main content
<!-- /main -->

<!-- aside -->
Sidebar
<!-- /aside -->

<!-- footer -->
Page footer
<!-- /footer -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_semantic_elements_with_images() -> None:
    """Test semantic elements containing images."""
    html = """
    <article>
        <header>
            <h1>Article with Image</h1>
            <img src="header.jpg" alt="Header image" />
        </header>
        <section>
            <p>Content with <img src="inline.jpg" alt="Inline image" /> inline image.</p>
        </section>
    </article>
    """
    expected = """<!-- article -->
<!-- header -->
Article with Image
==================

![Header image](header.jpg)
<!-- /header -->

<!-- section -->
Content with ![Inline image](inline.jpg) inline image.
<!-- /section -->
<!-- /article -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip()


def test_semantic_elements_with_emphasis() -> None:
    """Test semantic elements with emphasis and formatting."""
    html = """
    <section>
        <h2>Formatted Content</h2>
        <p>This is <strong>bold</strong> and <em>italic</em> text.</p>
        <p>Here's some <code>inline code</code> and <mark>highlighted</mark> text.</p>
    </section>
    """
    expected = """<!-- section -->
Formatted Content
-----------------

This is **bold** and *italic* text.

Here's some `inline code` and ==highlighted== text.
<!-- /section -->"""
    result = convert_to_markdown(html)
    assert result.strip() == expected.strip() 