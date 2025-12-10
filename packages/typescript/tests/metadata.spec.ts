import { rm, writeFile } from "node:fs/promises";
import { Readable } from "node:stream";
import { describe, expect, it } from "vitest";
import {
	convertWithMetadata,
	convertWithMetadataBuffer,
	convertFileWithMetadata,
	convertStreamWithMetadata,
	type JsMetadataConfig,
	type JsMetadataExtraction,
} from "../src/index";

const BASIC_HTML = `
<html lang="en">
	<head>
		<title>Test Article</title>
		<meta name="description" content="A test article about markdown">
		<meta name="author" content="Test Author">
		<meta name="keywords" content="markdown, html, conversion">
		<meta property="og:title" content="OG Title">
		<meta property="og:description" content="OG Description">
		<meta name="twitter:card" content="summary">
		<link rel="canonical" href="https://example.com/article">
	</head>
	<body>
		<h1 id="main-title">Main Title</h1>
		<h2>Section 1</h2>
		<p>Some text with <a href="/page">internal link</a> and <a href="https://example.com">external link</a>.</p>
		<h3>Subsection</h3>
		<p>More content with <a href="mailto:test@example.com">email link</a> and <a href="#section">anchor</a>.</p>
		<img src="https://example.com/image.jpg" alt="Test Image" title="Test Title">
		<img src="data:image/png;base64,iVBORw0KGgo=" alt="Embedded Image">
	</body>
</html>
`;

describe("html-to-markdown metadata extraction (TypeScript)", () => {
	it("extracts document metadata", () => {
		const result = convertWithMetadata(BASIC_HTML);

		expect(result).toHaveProperty("markdown");
		expect(result).toHaveProperty("metadata");

		const metadata = result.metadata;
		expect(metadata.document.title).toBe("Test Article");
		expect(metadata.document.description).toBe("A test article about markdown");
		expect(metadata.document.author).toBe("Test Author");
		expect(metadata.document.language).toBe("en");
		expect(metadata.document.canonical_url).toBe("https://example.com/article");

		// Keywords should be split on commas
		expect(metadata.document.keywords).toContain("markdown");
		expect(metadata.document.keywords).toContain("html");
		expect(metadata.document.keywords).toContain("conversion");

		// Open Graph metadata
		expect(metadata.document.open_graph).toHaveProperty("title", "OG Title");
		expect(metadata.document.open_graph).toHaveProperty("description", "OG Description");

		// Twitter Card metadata
		expect(metadata.document.twitter_card).toHaveProperty("card", "summary");
	});

	it("extracts header metadata with hierarchy", () => {
		const result = convertWithMetadata(BASIC_HTML);
		const headers = result.metadata.headers;

		expect(headers.length).toBeGreaterThanOrEqual(3);

		// H1
		const h1 = headers.find((h) => h.level === 1);
		expect(h1).toBeDefined();
		expect(h1?.text).toBe("Main Title");
		expect(h1?.id).toBe("main-title");

		// H2
		const h2 = headers.find((h) => h.level === 2);
		expect(h2).toBeDefined();
		expect(h2?.text).toBe("Section 1");

		// H3
		const h3 = headers.find((h) => h.level === 3);
		expect(h3).toBeDefined();
		expect(h3?.text).toBe("Subsection");
	});

	it("extracts and classifies links", () => {
		const result = convertWithMetadata(BASIC_HTML);
		const links = result.metadata.links;

		expect(links.length).toBeGreaterThanOrEqual(4);

		// Check link classification
		const internalLink = links.find((l) => l.href === "/page");
		expect(internalLink).toBeDefined();
		expect(internalLink?.link_type).toBe("internal");
		expect(internalLink?.text).toBe("internal link");

		const externalLink = links.find((l) => l.href === "https://example.com");
		expect(externalLink).toBeDefined();
		expect(externalLink?.link_type).toBe("external");

		const emailLink = links.find((l) => l.href === "mailto:test@example.com");
		expect(emailLink).toBeDefined();
		expect(emailLink?.link_type).toBe("email");
		expect(emailLink?.text).toBe("email link");

		const anchorLink = links.find((l) => l.href === "#section");
		expect(anchorLink).toBeDefined();
		expect(anchorLink?.link_type).toBe("anchor");
	});

	it("extracts image metadata with types", () => {
		const result = convertWithMetadata(BASIC_HTML);
		const images = result.metadata.images;

		expect(images.length).toBeGreaterThanOrEqual(2);

		// External image
		const externalImg = images.find((i) => i.src === "https://example.com/image.jpg");
		expect(externalImg).toBeDefined();
		expect(externalImg?.image_type).toBe("external");
		expect(externalImg?.alt).toBe("Test Image");
		expect(externalImg?.title).toBe("Test Title");

		// Data URI image
		const dataUriImg = images.find((i) => i.src.startsWith("data:image"));
		expect(dataUriImg).toBeDefined();
		expect(dataUriImg?.image_type).toBe("data_uri");
		expect(dataUriImg?.alt).toBe("Embedded Image");
	});

	it("respects metadata extraction config flags", () => {
		const config: JsMetadataConfig = {
			extract_headers: false,
			extract_links: false,
			extract_images: false,
			extract_structured_data: true,
			max_structured_data_size: 1_000_000,
		};

		const result = convertWithMetadata(BASIC_HTML, undefined, config);
		const metadata = result.metadata;

		// Document metadata is always extracted
		expect(metadata.document.title).toBe("Test Article");

		// But headers and links should be empty
		expect(metadata.headers).toHaveLength(0);
		expect(metadata.links).toHaveLength(0);
		expect(metadata.images).toHaveLength(0);
	});

	it("converts from Buffer without string allocation", () => {
		const buffer = Buffer.from(BASIC_HTML, "utf8");
		const result = convertWithMetadataBuffer(buffer);

		expect(result.markdown).toBeTruthy();
		expect(result.metadata.document.title).toBe("Test Article");
		expect(result.metadata.headers.length).toBeGreaterThan(0);
	});

	it("converts from file", async () => {
		const path = "tmp-metadata-test.html";
		await writeFile(path, BASIC_HTML, "utf8");

		try {
			const result = await convertFileWithMetadata(path);

			expect(result.markdown).toContain("Main Title");
			expect(result.metadata.document.title).toBe("Test Article");
			expect(result.metadata.document.author).toBe("Test Author");
		} finally {
			await rm(path, { force: true });
		}
	});

	it("converts from stream", async () => {
		const stream = Readable.from([BASIC_HTML]);
		const result = await convertStreamWithMetadata(stream);

		expect(result.markdown).toContain("Main Title");
		expect(result.metadata.document.title).toBe("Test Article");
		expect(result.metadata.headers.length).toBeGreaterThan(0);
	});

	it("handles HTML with minimal metadata", () => {
		const minimalHtml = "<h1>Title</h1><p>Content</p>";
		const result = convertWithMetadata(minimalHtml);

		// Document metadata should exist but mostly be empty
		expect(result.metadata.document.title).toBeUndefined();
		expect(result.metadata.document.description).toBeUndefined();

		// Should still extract the heading
		expect(result.metadata.headers.length).toBe(1);
		expect(result.metadata.headers[0].text).toBe("Title");
	});

	it("extracts markdown with metadata", () => {
		const result = convertWithMetadata(BASIC_HTML);

		// Verify markdown was generated
		expect(result.markdown).toContain("Main Title");
		expect(result.markdown).toContain("Section 1");
		expect(result.markdown).toContain("Subsection");
		expect(result.markdown).toContain("internal link");
	});

	it("handles structured data size limits", () => {
		const htmlWithLargeJson = `
		<html>
			<head>
				<script type="application/ld+json">
					${JSON.stringify({ "@type": "Article", content: "x".repeat(10000) })}
				</script>
			</head>
			<body><p>Test</p></body>
		</html>
		`;

		// With 1MB limit (default), should work
		const result1 = convertWithMetadata(htmlWithLargeJson);
		expect(result1).toHaveProperty("metadata");

		// With 100 byte limit, should respect it
		const config: JsMetadataConfig = {
			extract_headers: true,
			extract_links: true,
			extract_images: true,
			extract_structured_data: true,
			max_structured_data_size: 100,
		};
		const result2 = convertWithMetadata(htmlWithLargeJson, undefined, config);
		expect(result2).toHaveProperty("metadata");
	});

	it("handles special characters in metadata", () => {
		const specialHtml = `
		<html lang="fr">
			<head>
				<title>Café & Restaurant</title>
				<meta name="description" content="A guide to French cuisine">
			</head>
			<body>
				<h1>Délicieux!</h1>
				<a href="/résumé">Résumé</a>
			</body>
		</html>
		`;

		const result = convertWithMetadata(specialHtml);

		expect(result.metadata.document.title).toBe("Café & Restaurant");
		expect(result.metadata.document.language).toBe("fr");
		expect(result.metadata.headers[0].text).toBe("Délicieux!");
	});
});
