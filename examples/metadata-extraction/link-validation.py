#!/usr/bin/env python3
"""
Link Validation and Classification Example.

Demonstrates how to extract and classify links by type (anchor, internal, external,
email, phone), validate link structures, and identify potential issues.
"""

from html_to_markdown import convert_with_metadata


def analyze_links(html: str) -> dict:
    """Analyze and classify all links in the document."""
    _markdown, metadata = convert_with_metadata(html)
    links = metadata["links"]

    analysis = {
        "total_links": len(links),
        "by_type": {},
        "empty_links": [],
        "broken_anchors": [],
        "external_links": [],
        "email_links": [],
        "phone_links": [],
        "links_without_title": [],
    }

    # Initialize type counters
    for link in links:
        link_type = link["link_type"]
        analysis["by_type"][link_type] = analysis["by_type"].get(link_type, 0) + 1

    # Categorize links and check for issues
    for link in links:
        href = link["href"]
        text = link.get("text", "").strip()
        link_type = link["link_type"]
        title = link.get("title")

        # Check for empty link text
        if not text:
            analysis["empty_links"].append({"href": href, "text": "(empty)"})

        # Collect external links
        if link_type == "external":
            analysis["external_links"].append(
                {
                    "href": href,
                    "text": text,
                    "title": title,
                    "rel": link.get("rel", []),
                }
            )

        # Collect anchor/fragment links
        if link_type == "anchor":
            analysis["broken_anchors"].append(
                {
                    "href": href,
                    "text": text,
                    "fragment": href.lstrip("#"),
                }
            )

        # Collect email links
        if link_type == "email":
            email = href.replace("mailto:", "")
            analysis["email_links"].append(
                {
                    "email": email,
                    "text": text,
                }
            )

        # Collect phone links
        if link_type == "phone":
            phone = href.replace("tel:", "")
            analysis["phone_links"].append(
                {
                    "phone": phone,
                    "text": text,
                }
            )

        # Track links without title attribute
        if not title:
            analysis["links_without_title"].append({"href": href, "text": text})

    return analysis


def main() -> None:
    html = """
    <html>
      <head>
        <title>Web Standards Reference</title>
      </head>
      <body>
        <h1>Web Development Standards</h1>

        <h2 id="getting-started">Getting Started</h2>
        <p>Welcome! Here are some resources to get started:</p>

        <h2 id="html">HTML Standards</h2>
        <p>Learn about modern HTML standards:</p>
        <ul>
          <li><a href="https://html.spec.whatwg.org/" title="WHATWG HTML Standard">HTML Living Standard</a></li>
          <li><a href="https://www.w3.org/TR/html5/">W3C HTML5 Recommendation</a></li>
          <li><a href="/guides/semantic-html">Semantic HTML Guide</a></li>
        </ul>

        <h2 id="css">CSS Styling</h2>
        <p>CSS resources:</p>
        <ul>
          <li><a href="https://www.w3.org/Style/CSS/" title="W3C CSS Specifications">W3C CSS</a></li>
          <li><a href="/tutorials/css-flexbox">CSS Flexbox Tutorial</a></li>
          <li><a href="#getting-started">Back to Getting Started</a></li>
        </ul>

        <h2 id="javascript">JavaScript</h2>
        <p>JavaScript development:</p>
        <ul>
          <li><a href="https://tc39.es/">TC39 - ECMAScript Standard</a></li>
          <li><a href="../getting-started">Parent Page Guide</a></li>
          <li><a href="./relative-page.html">Relative Link</a></li>
        </ul>

        <h2 id="contact">Contact & Support</h2>
        <p>Get in touch:</p>
        <ul>
          <li><a href="mailto:support@example.com">Email Support</a></li>
          <li><a href="tel:+1-555-0123" title="Call our hotline">Call Us</a></li>
          <li><a href="">Empty Link (broken)</a></li>
          <li><a href="https://twitter.com/webstandards">Follow on Twitter</a></li>
        </ul>

        <h2 id="broken-anchor">Broken Anchor</h2>
        <p>Link to non-existent section: <a href="#non-existent-section">Jump to section</a></p>

        <h2 id="accessibility">Accessibility</h2>
        <p><a href="https://www.w3.org/WAI/">Web Accessibility Initiative</a></p>
      </body>
    </html>
    """

    analysis = analyze_links(html)

    # Summary
    for link_type in sorted(analysis["by_type"].keys()):
        analysis["by_type"][link_type]

    # External links
    if analysis["external_links"]:
        for link in analysis["external_links"]:
            if link["title"]:
                pass
            if link["rel"]:
                pass

    # Email links
    if analysis["email_links"]:
        for link in analysis["email_links"]:
            pass

    # Phone links
    if analysis["phone_links"]:
        for link in analysis["phone_links"]:
            pass

    # Anchor links (potential broken references)
    if analysis["broken_anchors"]:
        for link in analysis["broken_anchors"]:
            pass

    # Empty links (accessibility issue)
    if analysis["empty_links"]:
        for link in analysis["empty_links"]:
            pass

    # Links without title attribute
    if analysis["links_without_title"]:
        for link in analysis["links_without_title"][:5]:
            pass
        if len(analysis["links_without_title"]) > 5:
            pass


if __name__ == "__main__":
    main()
