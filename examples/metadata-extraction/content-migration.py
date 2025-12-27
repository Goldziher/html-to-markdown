#!/usr/bin/env python3
"""
Content Migration with Resource Manifest Example.

Demonstrates how to create a migration manifest documenting all external resources,
links, images, and metadata for content preservation during migrations.
"""

import json

from html_to_markdown import convert_with_metadata


def create_migration_manifest(html: str, document_id: str | None = None) -> dict:
    """Create a comprehensive migration manifest from HTML content."""
    markdown, metadata = convert_with_metadata(html)

    # Count resources
    external_images = [img for img in metadata["images"] if img["image_type"] == "external"]
    external_links = [link for link in metadata["links"] if link["link_type"] == "external"]
    json_ld_blocks = [s for s in metadata["structured_data"] if s["data_type"] == "json_ld"]

    return {
        "metadata": {
            "document_id": document_id,
            "title": metadata["document"].get("title"),
            "language": metadata["document"].get("language"),
            "canonical_url": metadata["document"].get("canonical_url"),
            "author": metadata["document"].get("author"),
            "description": metadata["document"].get("description"),
            "original_content_length": len(html),
            "converted_markdown_length": len(markdown),
        },
        "resources": {
            "total_headers": len(metadata["headers"]),
            "total_links": len(metadata["links"]),
            "total_images": len(metadata["images"]),
            "external_links": len(external_links),
            "external_images": len(external_images),
            "structured_data_blocks": len(metadata["structured_data"]),
            "json_ld_blocks": len(json_ld_blocks),
        },
        "external_assets": {
            "images": [
                {
                    "url": img["src"],
                    "alt": img.get("alt"),
                    "title": img.get("title"),
                    "dimensions": img.get("dimensions"),
                }
                for img in external_images
            ],
            "links": [
                {
                    "url": link["href"],
                    "text": link.get("text"),
                    "title": link.get("title"),
                    "rel": link.get("rel", []),
                }
                for link in external_links
            ],
        },
        "content": {
            "markdown": markdown,
            "headers": [
                {
                    "level": h["level"],
                    "text": h["text"],
                    "id": h.get("id"),
                }
                for h in metadata["headers"]
            ],
        },
        "social_metadata": {
            "open_graph": metadata["document"].get("open_graph", {}),
            "twitter_card": metadata["document"].get("twitter_card", {}),
        },
    }


def print_migration_report(manifest: dict) -> None:
    """Print a formatted migration report."""
    meta = manifest["metadata"]
    manifest["resources"]
    assets = manifest["external_assets"]

    # Document information
    if meta["document_id"]:
        pass

    # Resource summary

    # External images
    if assets["images"]:
        for _i, img in enumerate(assets["images"], 1):
            if img["alt"]:
                pass
            if img["dimensions"]:
                _w, _h = img["dimensions"]

    # External links
    if assets["links"]:
        for _i, link in enumerate(assets["links"], 1):
            if link["title"]:
                pass
            if link["rel"]:
                pass

    # Social metadata
    og = manifest["social_metadata"]["open_graph"]
    tw = manifest["social_metadata"]["twitter_card"]

    if og or tw:
        if og:
            for _key, _value in og.items():
                pass
        if tw:
            for _key, _value in tw.items():
                pass

    # Headers structure
    if manifest["content"]["headers"]:
        for header in manifest["content"]["headers"]:
            "  " * (header["level"] - 1)
            f" (#{header['id']})" if header["id"] else ""

    # Migration checklist


def main() -> None:
    html = """
    <html lang="en">
      <head>
        <title>Company Blog: Scaling with Rust</title>
        <meta name="description" content="Learn how to build scalable systems with Rust.">
        <meta name="author" content="Engineering Team">
        <link rel="canonical" href="https://blog.example.com/scaling-with-rust">
        <meta property="og:title" content="Scaling with Rust">
        <meta property="og:description" content="Production strategies for high-performance systems.">
        <meta property="og:image" content="https://cdn.example.com/blog/rust-hero.jpg">
        <meta name="twitter:card" content="summary_large_image">
        <meta name="twitter:creator" content="@engineering">
      </head>
      <body>
        <article>
          <h1 id="intro">Scaling Web Services with Rust</h1>
          <p>Published by the Engineering Team</p>

          <h2 id="why-rust">Why Choose Rust?</h2>
          <p>Rust combines performance with safety. Learn more at <a href="https://rust-lang.org">the Rust Foundation</a>.</p>

          <h2 id="architecture">Architecture Patterns</h2>
          <p>
            <img src="https://cdn.example.com/blog/architecture-diagram.png" alt="Microservices architecture diagram" width="800" height="600">
          </p>

          <h3 id="async-io">Asynchronous I/O</h3>
          <p>Use <code>tokio</code> for async runtime. Documentation: <a href="https://tokio.rs">tokio.rs</a></p>

          <h3 id="concurrency">Concurrency Patterns</h3>
          <p>Channel-based communication for thread safety.</p>

          <h2 id="case-studies">Case Studies</h2>
          <p>
            <img src="https://cdn.example.com/blog/companies-using-rust.jpg" alt="Companies successfully using Rust in production" width="1200" height="400">
          </p>

          <h3 id="discord">Discord's Migration</h3>
          <p>Read about <a href="https://discord.com/blog/using-rust-to-scale-discord">Discord's Rust migration</a>.</p>

          <h3 id="cloudflare">Cloudflare's Approach</h3>
          <p>See <a href="https://cloudflare.com/learning/performance/">Cloudflare's performance strategies</a>.</p>

          <h2 id="tools">Development Tools</h2>
          <ul>
            <li><a href="https://github.com/tokio-rs/tokio">Tokio</a> - Async runtime</li>
            <li><a href="https://serde.rs">Serde</a> - Serialization</li>
            <li><a href="https://prometheus.io">Prometheus</a> - Monitoring</li>
          </ul>

          <h2 id="resources">Additional Resources</h2>
          <p>
            Contact: <a href="mailto:rust@example.com">rust@example.com</a> or
            <a href="tel:+1-555-0123">call us</a>
          </p>

          <h2 id="related">Related Articles</h2>
          <p>
            <a href="/blog/rust-testing">Testing in Rust</a> |
            <a href="/blog/rust-performance">Performance Tuning</a> |
            <a href="#intro">Back to top</a>
          </p>
        </article>
      </body>
    </html>
    """

    # Create manifest
    manifest = create_migration_manifest(html, document_id="doc_001")

    # Print report
    print_migration_report(manifest)

    # Also save as JSON

    with open("migration-manifest.json", "w") as f:
        # Remove markdown content for cleaner JSON output
        manifest_for_json = {k: v for k, v in manifest.items() if k != "content"}
        manifest_for_json["content"] = {
            "headers": manifest["content"]["headers"],
            "note": "Markdown content omitted from JSON; see 'markdown' key in manifest",
        }
        json.dump(manifest_for_json, f, indent=2)


if __name__ == "__main__":
    main()
