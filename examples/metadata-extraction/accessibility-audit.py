#!/usr/bin/env python3
"""
Accessibility Audit Example.

Demonstrates how to check for common accessibility issues including:
- Images without alt text
- Empty link text
- Images without dimensions
- Missing link descriptions
"""

from html_to_markdown import convert_with_metadata


def audit_accessibility(html: str) -> dict:
    """Audit HTML for common accessibility issues."""
    _markdown, metadata = convert_with_metadata(html)

    audit = {
        "images_without_alt": [],
        "images_without_dimensions": [],
        "links_without_text": [],
        "empty_alt_text": [],
        "small_images_no_alt": [],
        "issue_count": 0,
        "total_checks": 0,
    }

    # Check images
    for img in metadata["images"]:
        src = img.get("src", "")
        alt = img.get("alt", "").strip()
        dimensions = img.get("dimensions")

        # Check for missing alt text
        if not alt:
            audit["images_without_alt"].append(
                {
                    "src": src,
                    "image_type": img.get("image_type"),
                    "severity": "critical",
                }
            )
            audit["issue_count"] += 1

        # Check for empty alt text (alt="" is also an issue for content images)
        if img.get("alt") == "":
            audit["empty_alt_text"].append({"src": src})
            audit["issue_count"] += 1

        # Check for missing dimensions
        if not dimensions:
            audit["images_without_dimensions"].append(
                {
                    "src": src,
                    "severity": "warning",
                }
            )

        # Check for small images (likely decorative) without proper marking
        if dimensions and dimensions[0] < 50 and dimensions[1] < 50 and (not alt or alt == ""):
            audit["small_images_no_alt"].append(
                {
                    "src": src,
                    "dimensions": dimensions,
                    "note": "Decorative image should have empty alt text",
                }
            )

        audit["total_checks"] += 1

    # Check links
    for link in metadata["links"]:
        text = link.get("text", "").strip()
        href = link.get("href", "")

        # Check for empty link text
        if not text or text == "":
            audit["links_without_text"].append(
                {
                    "href": href,
                    "severity": "critical",
                }
            )
            audit["issue_count"] += 1

        audit["total_checks"] += 1

    return audit


def generate_audit_report(html: str) -> str:
    """Generate a formatted accessibility audit report."""
    audit = audit_accessibility(html)

    lines = []
    lines.append("=" * 80)
    lines.append("ACCESSIBILITY AUDIT REPORT")
    lines.append("=" * 80)
    lines.append("")

    # Summary
    lines.append("SUMMARY")
    lines.append("-" * 80)
    lines.append(f"Issues found:        {audit['issue_count']}")
    lines.append(f"Total checks:        {audit['total_checks']}")
    severity = "PASS" if audit["issue_count"] == 0 else "FAIL"
    lines.append(f"Audit result:        {severity}")
    lines.append("")

    # Critical issues
    if audit["images_without_alt"]:
        lines.append("CRITICAL: Images Without Alt Text")
        lines.append("-" * 80)
        lines.append("Images without alt text are not accessible to screen readers.")
        lines.append("Every image should have descriptive alt text.\n")
        for img in audit["images_without_alt"]:
            lines.append(f"  Source:     {img['src']}")
            lines.append(f"  Type:       {img['image_type']}")
            lines.append("")

    if audit["links_without_text"]:
        lines.append("CRITICAL: Links Without Text")
        lines.append("-" * 80)
        lines.append("Links without text are not accessible to screen readers.")
        lines.append("Every link should have descriptive text content.\n")
        for link in audit["links_without_text"]:
            lines.append(f"  URL:        {link['href']}")
            lines.append("")

    if audit["empty_alt_text"]:
        lines.append("WARNING: Empty Alt Text")
        lines.append("-" * 80)
        lines.append("Images with empty alt text (alt='') are treated as decorative.")
        lines.append("Use this intentionally only for images that don't add meaning.\n")
        for img in audit["empty_alt_text"]:
            lines.append(f"  Source:     {img['src']}")
            lines.append("")

    if audit["images_without_dimensions"]:
        lines.append("WARNING: Images Without Dimensions")
        lines.append("-" * 80)
        lines.append("Images should have width and height attributes to prevent layout shift.")
        lines.append(f"Found {len(audit['images_without_dimensions'])} images without dimensions.\n")
        for img in audit["images_without_dimensions"][:5]:
            lines.append(f"  Source:     {img['src']}")
        if len(audit["images_without_dimensions"]) > 5:
            lines.append(f"  ... and {len(audit['images_without_dimensions']) - 5} more")
        lines.append("")

    if audit["small_images_no_alt"]:
        lines.append("INFO: Small Images (Decorative?)")
        lines.append("-" * 80)
        lines.append("Small images are often decorative. Consider using alt='' for true decorative images.\n")
        for img in audit["small_images_no_alt"]:
            lines.append(f"  Source:     {img['src']}")
            lines.append(f"  Size:       {img['dimensions'][0]}x{img['dimensions'][1]}px")
            lines.append(f"  Suggestion: {img['note']}")
            lines.append("")

    # No issues
    if audit["issue_count"] == 0:
        lines.append("✓ No critical accessibility issues found!")
        lines.append("")

    return "\n".join(lines)


def main() -> None:
    html = """
    <html lang="en">
      <head>
        <title>Product Gallery</title>
      </head>
      <body>
        <h1>Our Products</h1>

        <h2>Featured Items</h2>
        <img src="https://example.com/banner.jpg" alt="Summer Sale Banner" width="800" height="300">

        <h3>Electronics</h3>
        <p>
          <img src="https://example.com/camera.jpg" alt="Professional camera with interchangeable lenses" width="400" height="300">
          <a href="/cameras">View Camera Collection</a>
        </p>

        <h3>Accessories</h3>
        <p>
          <img src="https://example.com/headphones.jpg" width="300" height="300">
          Wireless headphones available in multiple colors.
          <a href="/headphones">Learn More</a>
        </p>

        <h3>Decorative Elements</h3>
        <p>
          <img src="data:image/gif;base64,R0lGODlhAQABAIAAAP///wAAACH5BAEAAAAALAAAAAABAAEAAAICRAEAOw==" alt="" width="1" height="1">
          Spacer pixel (correctly marked as decorative)
        </p>

        <h2>Quick Links</h2>
        <ul>
          <li><a href="/">Home</a></li>
          <li><a href="/about">About Us</a></li>
          <li><a href=""></a></li>
          <li><a href="/contact">Contact</a></li>
        </ul>

        <h2>Social Media</h2>
        <p>Follow us:
          <a href="https://twitter.com/company" title="Twitter">Twitter</a>
          <a href="https://facebook.com/company">Facebook</a>
          <a href="https://instagram.com/company">Instagram</a>
        </p>

        <h2>Gallery</h2>
        <div>
          <img src="https://example.com/photo1.jpg" width="200" height="200">
          <img src="https://example.com/photo2.jpg" alt="Team photo from 2025" width="200" height="200">
          <img src="https://example.com/photo3.jpg">
        </div>

        <h2>Support</h2>
        <p>Need help? <a href="mailto:support@example.com">Email our support team</a></p>
      </body>
    </html>
    """

    generate_audit_report(html)


if __name__ == "__main__":
    main()
