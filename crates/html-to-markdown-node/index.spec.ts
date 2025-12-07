import { Buffer } from "node:buffer";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
	convert,
	convertBuffer,
	convertBufferWithOptionsHandle,
	convertInlineImagesBuffer,
	convertWithInlineImages,
	convertWithOptionsHandle,
	createConversionOptionsHandle,
	JsCodeBlockStyle,
	JsHeadingStyle,
	JsHighlightStyle,
	JsListIndentType,
	JsNewlineStyle,
	JsPreprocessingPreset,
	JsWhitespaceMode,
} from "./index.js";

// Helper to load test documents
const loadTestDoc = (path: string): string => {
	const fullPath = join(__dirname, "../../test_documents", path);
	return readFileSync(fullPath, "utf-8");
};

describe("html-to-markdown-node - NAPI-RS Bindings", () => {
	describe("Basic Conversion", () => {
		it("should convert simple HTML to markdown", () => {
			const html = "<h1>Hello World</h1>";
			const markdown = convert(html);
			expect(markdown).toContain("Hello World");
		});

		it("should convert with null options", () => {
			const html = "<h1>Test</h1>";
			const markdown = convert(html, null);
			expect(markdown).toContain("Test");
		});

		it("should convert with undefined options", () => {
			const html = "<h1>Test</h1>";
			const markdown = convert(html, undefined);
			expect(markdown).toContain("Test");
		});

		it("should handle complex HTML", () => {
			const html = `
        <h1>Title</h1>
        <p>Paragraph with <strong>bold</strong> and <em>italic</em>.</p>
        <ul>
          <li>Item 1</li>
          <li>Item 2</li>
        </ul>
      `;
			const markdown = convert(html);
			expect(markdown).toContain("Title");
			expect(markdown).toMatch(/\*\*bold\*\*/);
			expect(markdown).toMatch(/\*italic\*/);
			expect(markdown).toMatch(/-\s+Item 1/);
		});
	});

	describe("Heading Styles", () => {
		it("should use ATX style", () => {
			const html = "<h1>Test</h1><h2>Subtest</h2>";
			const markdown = convert(html, { headingStyle: JsHeadingStyle.Atx });
			expect(markdown).toMatch(/^#\s+Test/m);
			expect(markdown).toMatch(/^##\s+Subtest/m);
		});

		it("should use underlined style", () => {
			const html = "<h1>Test</h1><h2>Subtest</h2>";
			const markdown = convert(html, {
				headingStyle: JsHeadingStyle.Underlined,
			});
			expect(markdown).toMatch(/Test\n={4,}/);
			expect(markdown).toMatch(/Subtest\n-{7,}/);
		});

		it("should use ATX closed style", () => {
			const html = "<h1>Test</h1>";
			const markdown = convert(html, {
				headingStyle: JsHeadingStyle.AtxClosed,
			});
			expect(markdown).toMatch(/#\s+Test\s+#/);
		});
	});

	describe("Option Handles", () => {
		it("should reuse parsed options across conversions", () => {
			const handle = createConversionOptionsHandle({
				headingStyle: JsHeadingStyle.AtxClosed,
			});
			const markdown = convertWithOptionsHandle("<h1>Reusable</h1>", handle);
			expect(markdown).toContain("# Reusable #");
		});

		it("should support default options via handles", () => {
			const handle = createConversionOptionsHandle();
			const markdown = convertWithOptionsHandle("<p>Default</p>", handle);
			expect(markdown).toContain("Default");
		});
	});

	describe("Buffer conversions", () => {
		it("should convert buffers without UTF-16 copies", () => {
			const html = Buffer.from("<h1>Buffer</h1>");
			const markdown = convertBuffer(html);
			expect(markdown).toContain("Buffer");
		});

		it("should convert buffers with handles", () => {
			const handle = createConversionOptionsHandle({
				headingStyle: JsHeadingStyle.Atx,
			});
			const html = Buffer.from("<h1>Buffered</h1>");
			const markdown = convertBufferWithOptionsHandle(html, handle);
			expect(markdown).toMatch(/^#\s+Buffered/m);
		});

		it("should reject non UTF-8 buffers", () => {
			const invalid = Buffer.from([0xff, 0xfe, 0xfd]);
			expect(() => convertBuffer(invalid)).toThrow(/UTF-8/);
		});
	});

	describe("Code Block Styles", () => {
		it("should use backticks for code blocks", () => {
			const html = "<pre><code>function test() {}</code></pre>";
			const markdown = convert(html, {
				codeBlockStyle: JsCodeBlockStyle.Backticks,
			});
			expect(markdown).toMatch(/```/);
		});

		it("should use tildes for code blocks", () => {
			const html = "<pre><code>function test() {}</code></pre>";
			const markdown = convert(html, {
				codeBlockStyle: JsCodeBlockStyle.Tildes,
			});
			expect(markdown).toMatch(/~~~/);
		});

		it("should use indented style for code blocks", () => {
			const html = "<pre><code>function test() {}</code></pre>";
			const markdown = convert(html, {
				codeBlockStyle: JsCodeBlockStyle.Indented,
			});
			expect(markdown).toMatch(/^\s{4,}function test/m);
		});

		it("should respect code language option", () => {
			const html = "<pre><code>function test() {}</code></pre>";
			const markdown = convert(html, {
				codeBlockStyle: JsCodeBlockStyle.Backticks,
				codeLanguage: "javascript",
			});
			expect(markdown).toMatch(/```javascript/);
		});
	});

	describe("List Options", () => {
		it("should handle spaces for list indentation", () => {
			const html = "<ul><li>Item 1<ul><li>Nested</li></ul></li></ul>";
			const markdown = convert(html, {
				listIndentType: JsListIndentType.Spaces,
				listIndentWidth: 4,
			});
			expect(markdown).toContain("Item 1");
			expect(markdown).toContain("Nested");
		});

		it("should handle tabs for list indentation", () => {
			const html = "<ul><li>Item</li></ul>";
			const markdown = convert(html, {
				listIndentType: JsListIndentType.Tabs,
			});
			expect(markdown).toContain("Item");
		});

		it("should respect custom bullets", () => {
			const html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
			const markdown = convert(html, {
				bullets: "+*-",
			});
			expect(markdown).toMatch(/[+*-]\s+Item/);
		});
	});

	describe("Text Formatting Options", () => {
		it("should respect strong/em symbol", () => {
			const html = "<strong>bold</strong> <em>italic</em>";
			const markdown = convert(html, {
				strongEmSymbol: "_",
			});
			expect(markdown).toMatch(/__bold__/);
			expect(markdown).toMatch(/_italic_/);
		});

		it("should escape asterisks when enabled", () => {
			const html = "<p>text * with * asterisks</p>";
			const markdown = convert(html, {
				escapeAsterisks: true,
			});
			expect(markdown).toMatch(/\\\*/);
		});

		it("should escape underscores when enabled", () => {
			const html = "<p>text_with_underscores</p>";
			const markdown = convert(html, {
				escapeUnderscores: true,
			});
			expect(markdown).toMatch(/\\_/);
		});

		it("should escape misc markdown characters", () => {
			const html = "<p>text [with] (special) {chars}</p>";
			const markdown = convert(html, {
				escapeMisc: true,
			});
			expect(markdown).toMatch(/\\\[/);
		});

		it("should escape ASCII punctuation", () => {
			const html = "<p>text! with? punctuation.</p>";
			const markdown = convert(html, {
				escapeAscii: true,
			});
			expect(markdown).toMatch(/\\!/);
		});
	});

	describe("Newline Styles", () => {
		it("should use spaces for newlines", () => {
			const html = "<p>Line 1<br>Line 2</p>";
			const markdown = convert(html, {
				newlineStyle: JsNewlineStyle.Spaces,
			});
			expect(markdown).toMatch(/\s{2}\n/);
		});

		it("should use backslash for newlines", () => {
			const html = "<p>Line 1<br>Line 2</p>";
			const markdown = convert(html, {
				newlineStyle: JsNewlineStyle.Backslash,
			});
			expect(markdown).toMatch(/\\\n/);
		});
	});

	describe("Table Options", () => {
		it("should use <br> in tables when enabled", () => {
			const html = "<table><tr><td>Line 1<br>Line 2</td></tr></table>";
			const markdown = convert(html, {
				brInTables: true,
			});
			expect(markdown).toContain("<br>");
		});

		it("should handle hOCR spatial tables", () => {
			// Load hOCR test document
			const hocr = loadTestDoc("test_data/hocr/comprehensive/valid_file.hocr");
			const markdown = convert(hocr, {
				hocrSpatialTables: true,
			});
			expect(markdown.length).toBeGreaterThan(0);
		});
	});

	describe("Highlight Styles", () => {
		it("should use double equal for mark elements", () => {
			const html = "<mark>highlighted</mark>";
			const markdown = convert(html, {
				highlightStyle: JsHighlightStyle.DoubleEqual,
			});
			expect(markdown).toMatch(/==highlighted==/);
		});

		it("should use HTML for mark elements", () => {
			const html = "<mark>highlighted</mark>";
			const markdown = convert(html, {
				highlightStyle: JsHighlightStyle.Html,
			});
			expect(markdown).toContain("<mark>highlighted</mark>");
		});

		it("should use bold for mark elements", () => {
			const html = "<mark>highlighted</mark>";
			const markdown = convert(html, {
				highlightStyle: JsHighlightStyle.Bold,
			});
			expect(markdown).toMatch(/\*\*highlighted\*\*/);
		});

		it("should use none for mark elements", () => {
			const html = "<mark>highlighted</mark>";
			const markdown = convert(html, {
				highlightStyle: JsHighlightStyle.None,
			});
			expect(markdown.trim()).toBe("highlighted");
		});
	});

	describe("Whitespace Handling", () => {
		it("should normalize whitespace", () => {
			const html = "<p>text   with    extra     spaces</p>";
			const markdown = convert(html, {
				whitespaceMode: JsWhitespaceMode.Normalized,
			});
			expect(markdown).toMatch(/text with extra spaces/);
		});

		it("should preserve strict whitespace", () => {
			const html = "<pre>text   with    spacing</pre>";
			const markdown = convert(html, {
				whitespaceMode: JsWhitespaceMode.Strict,
			});
			expect(markdown).toContain("   ");
		});

		it("should strip newlines when enabled", () => {
			const html = "<p>text\nwith\nnewlines</p>";
			const markdown = convert(html, {
				stripNewlines: true,
			});
			expect(markdown).not.toMatch(/text\nwith/);
		});
	});

	describe("Text Wrapping", () => {
		it("should accept wrap options", () => {
			const longText = "a".repeat(200);
			const html = `<p>${longText}</p>`;
			const markdown = convert(html, {
				wrap: true,
				wrapWidth: 80,
			});
			// Just verify it doesn't error and returns output
			expect(markdown.length).toBeGreaterThan(0);
			expect(markdown).toContain("a");
		});
	});

	describe("Special Elements", () => {
		it("should handle subscript", () => {
			const html = "<p>H<sub>2</sub>O</p>";
			const markdown = convert(html, {
				subSymbol: "~",
			});
			expect(markdown).toContain("~2~");
		});

		it("should handle superscript", () => {
			const html = "<p>E=mc<sup>2</sup></p>";
			const markdown = convert(html, {
				supSymbol: "^",
			});
			expect(markdown).toContain("^2^");
		});

		it("should handle autolinks option", () => {
			const html = "<p>Visit <a href='https://example.com'>https://example.com</a></p>";
			const markdown = convert(html, {
				autolinks: true,
			});
			expect(markdown).toContain("example.com");
		});

		it("should add default title when missing", () => {
			const html = "<html><body><h2>Subtitle</h2></body></html>";
			const markdown = convert(html, {
				defaultTitle: true,
			});
			expect(markdown.length).toBeGreaterThan(0);
		});
	});

	describe("Preprocessing", () => {
		it("should remove navigation elements", () => {
			const html = `
        <nav><a href="#">Menu</a></nav>
        <main><h1>Content</h1></main>
      `;
			const markdown = convert(html, {
				preprocessing: {
					enabled: true,
					removeNavigation: true,
				},
			});
			expect(markdown).toContain("Content");
			expect(markdown).not.toContain("Menu");
		});

		it("should remove form elements", () => {
			const html = `
        <h1>Title</h1>
        <form><input type="text"></form>
      `;
			const markdown = convert(html, {
				preprocessing: {
					enabled: true,
					removeForms: true,
				},
			});
			expect(markdown).toContain("Title");
			expect(markdown).not.toContain("input");
		});

		it("should use minimal preset", () => {
			const html = "<nav>Nav</nav><main>Main</main>";
			const markdown = convert(html, {
				preprocessing: {
					enabled: true,
					preset: JsPreprocessingPreset.Minimal,
				},
			});
			expect(markdown.length).toBeGreaterThan(0);
		});

		it("should use standard preset", () => {
			const html = "<nav>Nav</nav><main>Main</main>";
			const markdown = convert(html, {
				preprocessing: {
					enabled: true,
					preset: JsPreprocessingPreset.Standard,
				},
			});
			expect(markdown.length).toBeGreaterThan(0);
		});

		it("should use aggressive preset", () => {
			const html = "<nav>Nav</nav><main>Main</main>";
			const markdown = convert(html, {
				preprocessing: {
					enabled: true,
					preset: JsPreprocessingPreset.Aggressive,
				},
			});
			expect(markdown.length).toBeGreaterThan(0);
		});
	});

	describe("Advanced Options", () => {
		it("should convert as inline", () => {
			const html = "<div><p>Block</p></div>";
			const markdown = convert(html, {
				convertAsInline: true,
			});
			expect(markdown.trim()).toBe("Block");
		});

		it("should accept stripTags option", () => {
			const html = "<p>Keep this</p><div>Content</div>";
			const markdown = convert(html, {
				stripTags: ["div"],
			});
			expect(markdown).toContain("Keep this");
			// Just verify the option is accepted and doesn't error
			expect(markdown.length).toBeGreaterThan(0);
		});

		it("should keep inline images in specified elements", () => {
			const png =
				"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
			const html = `<a><img src="data:image/png;base64,${png}" alt="img"></a>`;
			const markdown = convert(html, {
				keepInlineImagesIn: ["a"],
			});
			expect(markdown).toContain("![img]");
		});

		it("should handle encoding option", () => {
			const html = "<p>Test</p>";
			const markdown = convert(html, {
				encoding: "utf-8",
			});
			expect(markdown).toContain("Test");
		});

		it("should enable debug mode", () => {
			const html = "<p>Test</p>";
			const markdown = convert(html, {
				debug: true,
			});
			expect(markdown).toContain("Test");
		});

		it("should extract metadata", () => {
			const html = "<html><head><title>Page Title</title></head><body>Content</body></html>";
			const markdown = convert(html, {
				extractMetadata: true,
			});
			expect(markdown.length).toBeGreaterThan(0);
		});
	});

	describe("Inline Images", () => {
		it("should extract inline images", () => {
			const png =
				"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
			const html = `<img src="data:image/png;base64,${png}" alt="Red Pixel">`;

			const result = convertWithInlineImages(html, undefined, {
				maxDecodedSizeBytes: 1024n,
				inferDimensions: true,
			});

			expect(result.markdown).toContain("Red Pixel");
			expect(result.inlineImages).toHaveLength(1);
			expect(result.inlineImages[0].format).toBe("png");
			expect(result.inlineImages[0].data).toBeInstanceOf(Buffer);
			expect(result.warnings).toHaveLength(0);
		});

		it("should handle invalid base64", () => {
			const html = '<img src="data:image/png;base64,invalid!!!" alt="Broken">';
			const result = convertWithInlineImages(html);
			expect(result.markdown).toContain("Broken");
			expect(result.inlineImages).toHaveLength(0);
			expect(result.warnings.length).toBeGreaterThan(0);
		});

		it("should extract SVG elements", () => {
			const html = '<svg><circle cx="10" cy="10" r="5"/></svg>';
			const result = convertWithInlineImages(html, undefined, {
				captureSvg: true,
			});
			const hasSvg = result.inlineImages.some((img) => img.format === "svg");
			expect(hasSvg).toBe(true);
		});

		it("should respect maxDecodedSizeBytes", () => {
			const png =
				"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
			const html = `<img src="data:image/png;base64,${png}" alt="img">`;
			const result = convertWithInlineImages(html, undefined, {
				maxDecodedSizeBytes: 1n, // Very small limit
			});
			// Should warn or not extract due to size
			expect(result.warnings.length > 0 || result.inlineImages.length === 0).toBe(true);
		});

		it("should use filename prefix", () => {
			const png =
				"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
			const html = `<img src="data:image/png;base64,${png}">`;
			const result = convertWithInlineImages(html, undefined, {
				filenamePrefix: "test_",
			});
			if (result.inlineImages.length > 0 && result.inlineImages[0].filename) {
				expect(result.inlineImages[0].filename).toMatch(/^test_/);
			}
		});

		it("should capture SVG when enabled", () => {
			const html = '<svg><rect width="10" height="10"/></svg>';
			const result = convertWithInlineImages(html, undefined, {
				captureSvg: true,
			});
			expect(result.inlineImages.some((img) => img.format === "svg")).toBe(true);
		});

		it("should not capture SVG when disabled", () => {
			const html = '<svg><rect width="10" height="10"/></svg>';
			const result = convertWithInlineImages(html, undefined, {
				captureSvg: false,
			});
			expect(result.inlineImages.some((img) => img.format === "svg")).toBe(false);
		});

		it("should support buffer inputs for inline images", () => {
			const png =
				"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
			const html = Buffer.from(`<img src="data:image/png;base64,${png}" alt="buf">`);
			const result = convertInlineImagesBuffer(html, undefined, {
				maxDecodedSizeBytes: 2048n,
			});
			expect(result.markdown).toContain("buf");
			expect(result.inlineImages).toHaveLength(1);
		});
	});

	describe("Edge Cases", () => {
		it("should handle empty HTML", () => {
			const markdown = convert("");
			expect(markdown).toBe("");
		});

		it("should handle HTML with only whitespace", () => {
			const markdown = convert("   \n   \t   ");
			expect(markdown.trim()).toBe("");
		});

		it("should handle malformed HTML gracefully", () => {
			const html = "<h1>Unclosed tag<p>Paragraph";
			const markdown = convert(html);
			expect(markdown).toContain("Unclosed tag");
			expect(markdown).toContain("Paragraph");
		});

		it("should handle special characters", () => {
			const html = "<p>Test & &lt;special&gt; characters</p>";
			const markdown = convert(html);
			expect(markdown).toContain("&");
			expect(markdown).toContain("special");
		});

		it("should handle unicode", () => {
			const html = "<p>Hello 世界 🌍</p>";
			const markdown = convert(html);
			expect(markdown).toContain("世界");
			expect(markdown).toContain("🌍");
		});

		it("should handle deeply nested elements", () => {
			let html = "<div>";
			for (let i = 0; i < 50; i++) {
				html += "<div>";
			}
			html += "Content";
			for (let i = 0; i < 50; i++) {
				html += "</div>";
			}
			html += "</div>";
			const markdown = convert(html);
			expect(markdown).toContain("Content");
		});

		it("should handle large documents", () => {
			const doc = loadTestDoc("html/wikipedia/large_rust.html");
			const markdown = convert(doc);
			expect(markdown.length).toBeGreaterThan(1000);
		});
	});

	describe("Performance", () => {
		it("should handle rapid conversions", () => {
			const html = "<h1>Test</h1><p>Content</p>";
			const iterations = 1000;

			const start = Date.now();
			for (let i = 0; i < iterations; i++) {
				convert(html);
			}
			const elapsed = Date.now() - start;
			const opsPerSec = Math.round((iterations / elapsed) * 1000);

			// Should handle at least 10k ops/sec (very conservative)
			expect(opsPerSec).toBeGreaterThan(10000);
		});

		it("should handle large document conversion", () => {
			const doc = loadTestDoc("html/wikipedia/large_rust.html");
			const start = Date.now();
			convert(doc);
			const elapsed = Date.now() - start;
			// Should complete in reasonable time (< 100ms for large doc)
			expect(elapsed).toBeLessThan(100);
		});
	});

	describe("Real World Documents", () => {
		it("should convert small Wikipedia document", () => {
			const doc = loadTestDoc("html/wikipedia/small_html.html");
			const markdown = convert(doc);
			expect(markdown.length).toBeGreaterThan(10);
		});

		it("should convert medium Wikipedia document", () => {
			const doc = loadTestDoc("html/wikipedia/medium_python.html");
			const markdown = convert(doc);
			expect(markdown.length).toBeGreaterThan(100);
		});

		it("should convert Wikipedia lists timeline", () => {
			const doc = loadTestDoc("html/wikipedia/lists_timeline.html");
			const markdown = convert(doc);
			expect(markdown).toMatch(/-\s+/); // Should have list items
		});

		it("should convert Wikipedia tables", () => {
			const doc = loadTestDoc("html/wikipedia/tables_countries.html");
			const markdown = convert(doc);
			expect(markdown).toMatch(/\|/); // Should have table delimiters
		});
	});
});
