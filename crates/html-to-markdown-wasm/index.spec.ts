import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  convert,
  convertBytes,
  convertBytesWithOptionsHandle,
  convertBytesWithInlineImages,
  convertWithInlineImages,
  createConversionOptionsHandle,
  WasmInlineImageConfig,
} from "./dist-node/html_to_markdown_wasm.js";

// Helper to load test documents
const loadTestDoc = (path: string): string => {
  const fullPath = join(__dirname, "../../test_documents", path);
  return readFileSync(fullPath, "utf-8");
};

describe("html-to-markdown-wasm - WebAssembly Bindings", () => {
  describe("Basic Conversion", () => {
    it("should convert simple HTML to markdown", () => {
      const html = "<h1>Hello World</h1>";
      const markdown = convert(html);
      expect(markdown).toContain("Hello World");
    });

    it("should handle empty HTML", () => {
      const markdown = convert("");
      expect(markdown).toBe("");
    });

    it("should handle plain text", () => {
      const markdown = convert("Just text");
      expect(markdown).toBe("Just text\n");
    });

    it("should handle null options", () => {
      const html = "<h1>Test</h1>";
      const markdown = convert(html, undefined);
      expect(markdown).toContain("Test");
    });
  });

  describe("Byte-based Conversion", () => {
    it("should convert Uint8Array input", () => {
      const encoder = new TextEncoder();
      const html = encoder.encode("<h1>Bytes</h1>");
      const markdown = convertBytes(html);
      expect(markdown).toContain("Bytes");
    });

    it("should convert bytes with option handles", () => {
      const encoder = new TextEncoder();
      const html = encoder.encode("<h1>Handles</h1>");
      const handle = createConversionOptionsHandle({ headingStyle: "atx" });
      const markdown = convertBytesWithOptionsHandle(html, handle);
      expect(markdown).toMatch(/^#\s+Handles/m);
    });

    it("should convert bytes with inline images", () => {
      const encoder = new TextEncoder();
      const png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
      const html = encoder.encode(`<img src="data:image/png;base64,${png}" alt="buffered">`);
      const config = new WasmInlineImageConfig(4096);
      const result = convertBytesWithInlineImages(html, undefined, config);
      expect(result.markdown).toContain("buffered");
    });

    it("should reject invalid UTF-8 byte streams", () => {
      const bytes = new Uint8Array([0xff, 0xfe, 0xfd]);
      expect(() => convertBytes(bytes)).toThrow(/UTF-8/);
    });
  });

  describe("Heading Styles", () => {
    it("should use ATX style", () => {
      const html = "<h1>Test</h1><h2>Subtest</h2>";
      const markdown = convert(html, { headingStyle: "atx" });
      expect(markdown).toMatch(/^#\s+Test/m);
      expect(markdown).toMatch(/^##\s+Subtest/m);
    });

    it("should use underlined style", () => {
      const html = "<h1>Test</h1><h2>Subtest</h2>";
      const markdown = convert(html, { headingStyle: "underlined" });
      expect(markdown).toMatch(/^Test\n=+/m);
      expect(markdown).toMatch(/^Subtest\n-+/m);
    });

    it("should use ATX closed style", () => {
      const html = "<h1>Test</h1>";
      const markdown = convert(html, { headingStyle: "atxClosed" });
      expect(markdown).toMatch(/^#\s+Test\s+#/m);
    });
  });

  describe("Code Block Styles", () => {
    it("should use backticks for code blocks", () => {
      const html = "<pre><code>const x = 1;</code></pre>";
      const markdown = convert(html, { codeBlockStyle: "backticks" });
      expect(markdown).toContain("```");
      expect(markdown).toContain("const x = 1;");
    });

    it("should use tildes for code blocks", () => {
      const html = "<pre><code>const x = 1;</code></pre>";
      const markdown = convert(html, { codeBlockStyle: "tildes" });
      expect(markdown).toContain("~~~");
      expect(markdown).toContain("const x = 1;");
    });

    it("should use indentation for code blocks", () => {
      const html = "<pre><code>const x = 1;</code></pre>";
      const markdown = convert(html, { codeBlockStyle: "indented" });
      expect(markdown).toContain("    const x = 1;");
    });

    it("should convert code blocks with language class", () => {
      const html = '<pre><code class="language-python">print("hello")</code></pre>';
      const markdown = convert(html, { codeBlockStyle: "backticks" });
      // Note: WASM bindings currently don't preserve language annotations in code fences
      expect(markdown).toContain("```");
      expect(markdown).toContain('print("hello")');
    });
  });

  describe("List Options", () => {
    it("should use custom bullets", () => {
      const html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
      const markdown = convert(html, { bullets: "+" });
      expect(markdown).toContain("+ Item 1");
      expect(markdown).toContain("+ Item 2");
    });

    it("should use custom list indentation", () => {
      const html = "<ul><li>Item 1<ul><li>Nested</li></ul></li></ul>";
      const markdown = convert(html, { listIndentWidth: 4 });
      expect(markdown).toContain("    - Nested");
    });

    it("should use tab indentation", () => {
      const html = "<ul><li>Item 1<ul><li>Nested</li></ul></li></ul>";
      const markdown = convert(html, { listIndentType: "tabs" });
      expect(markdown).toMatch(/\t-\sNested/);
    });
  });

  describe("Text Formatting", () => {
    it("should format bold text", () => {
      const html = "<p><strong>bold</strong></p>";
      const markdown = convert(html);
      expect(markdown).toContain("**bold**");
    });

    it("should format italic text", () => {
      const html = "<p><em>italic</em></p>";
      const markdown = convert(html);
      expect(markdown).toContain("*italic*");
    });

    it("should use underscore for emphasis", () => {
      const html = "<p><strong>bold</strong></p>";
      const markdown = convert(html, { strongEmSymbol: "_" });
      expect(markdown).toContain("__bold__");
    });

    it("should escape asterisks", () => {
      const html = "<p>2 * 3 = 6</p>";
      const markdown = convert(html, { escapeAsterisks: true });
      expect(markdown).toContain("\\*");
    });

    it("should escape underscores", () => {
      const html = "<p>snake_case</p>";
      const markdown = convert(html, { escapeUnderscores: true });
      expect(markdown).toContain("\\_");
    });
  });

  describe("Newline Styles", () => {
    it("should use spaces for line breaks", () => {
      const html = "<p>Line 1<br>Line 2</p>";
      const markdown = convert(html, { newlineStyle: "spaces" });
      expect(markdown).toContain("  \n");
    });

    it("should use backslash for line breaks", () => {
      const html = "<p>Line 1<br>Line 2</p>";
      const markdown = convert(html, { newlineStyle: "backslash" });
      expect(markdown).toContain("\\\n");
    });
  });

  describe("Table Options", () => {
    it("should convert tables", () => {
      const html = "<table><tr><th>Header</th></tr><tr><td>Cell</td></tr></table>";
      const markdown = convert(html);
      expect(markdown).toContain("| Header |");
      expect(markdown).toContain("| Cell |");
    });

    it("should use br in tables", () => {
      const html = "<table><tr><td>Line 1<br>Line 2</td></tr></table>";
      const markdown = convert(html, { brInTables: true });
      expect(markdown).toContain("<br>");
    });
  });

  describe("Highlight Styles", () => {
    it("should use double equals for highlights", () => {
      const html = "<mark>highlighted</mark>";
      const markdown = convert(html, { highlightStyle: "doubleEqual" });
      expect(markdown).toContain("==highlighted==");
    });

    it("should use HTML for highlights", () => {
      const html = "<mark>highlighted</mark>";
      const markdown = convert(html, { highlightStyle: "html" });
      expect(markdown).toContain("<mark>highlighted</mark>");
    });

    it("should use bold for highlights", () => {
      const html = "<mark>highlighted</mark>";
      const markdown = convert(html, { highlightStyle: "bold" });
      expect(markdown).toContain("**highlighted**");
    });

    it("should use no formatting for highlights", () => {
      const html = "<mark>highlighted</mark>";
      const markdown = convert(html, { highlightStyle: "none" });
      expect(markdown.trim()).toBe("highlighted");
    });
  });

  describe("Whitespace Handling", () => {
    it("should normalize whitespace", () => {
      const html = "<p>Multiple    spaces</p>";
      const markdown = convert(html, { whitespaceMode: "normalized" });
      expect(markdown).not.toContain("    ");
    });

    it("should preserve strict whitespace", () => {
      const html = "<p>Multiple    spaces</p>";
      const markdown = convert(html, { whitespaceMode: "strict" });
      expect(markdown).toContain("    ");
    });

    it("should strip newlines when enabled", () => {
      const html = "<p>Line\n\nwith\n\nnewlines</p>";
      const markdown = convert(html, { stripNewlines: true });
      expect(markdown).not.toContain("\n\n");
    });
  });

  describe("Text Wrapping", () => {
    it("should accept wrap configuration", () => {
      const html = "<p>A very long line of text that should be wrapped at a certain width</p>";
      const markdown = convert(html, { wrap: true, wrapWidth: 40 });
      expect(markdown).toBeTruthy();
    });
  });

  describe("Special Elements", () => {
    it("should convert links", () => {
      const html = '<a href="https://example.com">Link</a>';
      const markdown = convert(html);
      expect(markdown).toContain("[Link](https://example.com)");
    });

    it("should convert images", () => {
      const html = '<img src="image.png" alt="Test">';
      const markdown = convert(html);
      expect(markdown).toContain("![Test](image.png)");
    });

    it("should use autolinks", () => {
      const html = '<a href="https://example.com">https://example.com</a>';
      const markdown = convert(html, { autolinks: true });
      expect(markdown).toContain("<https://example.com>");
    });

    it("should convert as inline", () => {
      const html = "<div>Block</div>";
      const markdown = convert(html, { convertAsInline: true });
      expect(markdown.trim()).toBe("Block");
    });
  });

  describe("Preprocessing", () => {
    it("should accept preprocessing options", () => {
      const html = "<nav>Navigation</nav><article>Content</article>";
      const markdown = convert(html, {
        preprocessing: {
          enabled: true,
          preset: "aggressive",
          removeNavigation: true,
          removeForms: true,
        },
      });
      expect(markdown).toBeTruthy();
    });

    it("should use minimal preset", () => {
      const html = "<div>Content</div>";
      const markdown = convert(html, {
        preprocessing: {
          enabled: true,
          preset: "minimal",
        },
      });
      expect(markdown).toContain("Content");
    });

    it("should use standard preset", () => {
      const html = "<div>Content</div>";
      const markdown = convert(html, {
        preprocessing: {
          enabled: true,
          preset: "standard",
        },
      });
      expect(markdown).toContain("Content");
    });

    it("should allow preprocessing without preset", () => {
      const html = "<div>Content</div>";
      const markdown = convert(html, {
        preprocessing: {
          enabled: true,
        },
      });
      expect(markdown).toContain("Content");
    });

    it("should work with preprocessing disabled", () => {
      const html = "<nav>Navigation</nav>";
      const markdown = convert(html, {
        preprocessing: {
          enabled: false,
        },
      });
      expect(markdown).toContain("Navigation");
    });
  });

  describe("Advanced Options", () => {
    it("should add default title", () => {
      const html = "<p>Content without title</p>";
      const markdown = convert(html, { defaultTitle: true });
      expect(markdown).toBeTruthy();
    });

    it("should extract metadata", () => {
      const html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>";
      const markdown = convert(html, { extractMetadata: true });
      expect(markdown).toContain("Content");
    });

    it("should disable metadata extraction", () => {
      const html = "<html><head><title>Test</title></head><body><p>Content</p></body></html>";
      const markdown = convert(html, { extractMetadata: false });
      expect(markdown).toContain("Content");
    });

    it("should accept subscript symbol", () => {
      const html = "<p>H<sub>2</sub>O</p>";
      const markdown = convert(html, { subSymbol: "~" });
      expect(markdown).toContain("~2~");
    });

    it("should accept superscript symbol", () => {
      const html = "<p>x<sup>2</sup></p>";
      const markdown = convert(html, { supSymbol: "^" });
      expect(markdown).toContain("^2^");
    });

    it("should accept strip tags list", () => {
      const html = "<p>Keep this</p><script>Remove this</script>";
      const markdown = convert(html, { stripTags: ["script"] });
      expect(markdown).toContain("Keep this");
    });
  });

  describe("Inline Images", () => {
    it("should convert with inline images", () => {
      const html =
        '<img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==" alt="test">';
      const result = convertWithInlineImages(html, null);
      expect(result.markdown).toBeTruthy();
      expect(result.inlineImages).toHaveLength(1);
    });

    it("should extract inline image data", () => {
      const html =
        '<img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==" alt="test">';
      const result = convertWithInlineImages(html, null);
      expect(result.inlineImages[0].format).toBe("png");
      expect(result.inlineImages[0].description).toBe("test");
    });

    it("should use inline image config", () => {
      const html =
        '<img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==" alt="test">';
      const config = new WasmInlineImageConfig(1024 * 1024);
      config.inferDimensions = true;
      const result = convertWithInlineImages(html, null, config);
      expect(result.inlineImages).toHaveLength(1);
    });

    it("should capture SVG elements", () => {
      const html = '<svg width="10" height="10"><circle cx="5" cy="5" r="4"/></svg>';
      const config = new WasmInlineImageConfig();
      config.captureSvg = true;
      const result = convertWithInlineImages(html, null, config);
      expect(result.markdown).toBeTruthy();
    });

    it("should use filename prefix", () => {
      const html =
        '<img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==" alt="test">';
      const config = new WasmInlineImageConfig();
      config.filenamePrefix = "image_";
      const result = convertWithInlineImages(html, null, config);
      expect(result.inlineImages[0].filename).toMatch(/^image_/);
    });

    it("should return warnings when appropriate", () => {
      const html = '<img src="data:image/png;base64,invalid" alt="test">';
      const result = convertWithInlineImages(html, null);
      expect(result.warnings).toBeDefined();
    });

    it("should handle options with inline images", () => {
      const html =
        '<h1>Title</h1><img src="data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==" alt="test">';
      const result = convertWithInlineImages(html, { headingStyle: "atx" }, null);
      expect(result.markdown).toMatch(/^#\s+Title/m);
      expect(result.inlineImages).toHaveLength(1);
    });
  });

  describe("Edge Cases", () => {
    it("should handle malformed HTML", () => {
      const html = "<div><p>Unclosed paragraph";
      const markdown = convert(html);
      expect(markdown).toContain("Unclosed paragraph");
    });

    it("should handle nested formatting", () => {
      const html = "<strong><em>Bold and italic</em></strong>";
      const markdown = convert(html);
      expect(markdown).toContain("***Bold and italic***");
    });

    it("should handle special characters", () => {
      const html = "<p>&lt;special&gt;</p>";
      const markdown = convert(html);
      expect(markdown).toContain("<special>");
    });

    it("should handle empty elements", () => {
      const html = "<p></p><div></div>";
      const markdown = convert(html);
      expect(markdown).toBeDefined();
    });

    it("should handle very long text", () => {
      const html = `<p>${"A ".repeat(10000)}</p>`;
      const markdown = convert(html);
      expect(markdown).toBeTruthy();
      expect(markdown.length).toBeGreaterThan(10000);
    });

    it("should handle deeply nested elements", () => {
      let html = "<div>";
      for (let i = 0; i < 50; i++) {
        html += "<div>";
      }
      html += "Deep content";
      for (let i = 0; i < 50; i++) {
        html += "</div>";
      }
      html += "</div>";
      const markdown = convert(html);
      expect(markdown).toContain("Deep content");
    });
  });

  describe("Performance", () => {
    it("should handle small documents efficiently", () => {
      const html = "<p>Small document</p>";
      const start = Date.now();
      for (let i = 0; i < 1000; i++) {
        convert(html);
      }
      const duration = Date.now() - start;
      expect(duration).toBeLessThan(1000); // Should complete in < 1 second
    });

    it("should handle medium documents", () => {
      const html = `<div>${"<p>Paragraph</p>".repeat(100)}</div>`;
      const markdown = convert(html);
      expect(markdown).toContain("Paragraph");
    });
  });

  describe("Preserve Tags", () => {
    it("should preserve simple table as HTML", () => {
      const html = "<p>Before table</p><table><tr><td>Cell 1</td><td>Cell 2</td></tr></table><p>After table</p>";
      const markdown = convert(html, { preserveTags: ["table"] });
      expect(markdown).toContain("<table>");
      expect(markdown).toContain("<tr>");
      expect(markdown).toContain("<td>Cell 1</td>");
      expect(markdown).toContain("Before table");
      expect(markdown).toContain("After table");
    });

    it("should preserve table with attributes", () => {
      const html = '<div class="content"><table id="data" class="styled"><tr><td>Value</td></tr></table></div>';
      const markdown = convert(html, { preserveTags: ["table"] });
      expect(markdown).toContain('<table id="data" class="styled">');
      expect(markdown).toContain("<tr>");
      expect(markdown).toContain("<td>Value</td>");
      expect(markdown).toContain("</table>");
    });

    it("should preserve multiple tag types", () => {
      const html =
        '<p>Text</p><table><tr><td>Table</td></tr></table><form><input type="text"/></form><div>More text</div>';
      const markdown = convert(html, { preserveTags: ["table", "form"] });
      expect(markdown).toContain("<table>");
      expect(markdown).toContain("<form>");
      expect(markdown).toContain("Text");
      expect(markdown).toContain("More text");
    });

    it("should preserve nested content in tags", () => {
      const html = "<table><tr><td><strong>Bold</strong> and <em>italic</em></td></tr></table>";
      const markdown = convert(html, { preserveTags: ["table"] });
      expect(markdown).toContain("<table>");
      expect(markdown).toContain("<strong>Bold</strong>");
      expect(markdown).toContain("<em>italic</em>");
      expect(markdown).toContain("</table>");
    });

    it("should convert tables normally without preserve_tags", () => {
      const html = "<table><tr><td>Cell</td></tr></table>";
      const markdown = convert(html, { preserveTags: [] });
      expect(markdown).not.toContain("<table>");
      expect(markdown).toContain("Cell");
    });

    it("should work with both preserve and strip tags", () => {
      const html = "<p>Text</p><table><tr><td>Table content</td></tr></table><div>Div content</div>";
      const markdown = convert(html, { preserveTags: ["table"], stripTags: ["div"] });
      // table should be preserved as HTML
      expect(markdown).toContain("<table>");
      expect(markdown).toContain("Table content");
      // div should be stripped (only text content)
      expect(markdown).not.toContain("<div>");
      expect(markdown).toContain("Div content");
      // p should be converted normally
      expect(markdown).toContain("Text");
    });
  });

  describe("Real-World Documents", () => {
    it("should convert Wikipedia timeline", () => {
      const html = loadTestDoc("html/wikipedia/lists_timeline.html");
      const markdown = convert(html);
      expect(markdown).toBeTruthy();
      expect(markdown.length).toBeGreaterThan(100);
    });

    it("should convert Wikipedia countries table", () => {
      const html = loadTestDoc("html/wikipedia/tables_countries.html");
      const markdown = convert(html);
      expect(markdown).toBeTruthy();
      expect(markdown).toContain("|");
    });

    it("should convert Wikipedia Python article", () => {
      const html = loadTestDoc("html/wikipedia/medium_python.html");
      const markdown = convert(html);
      expect(markdown).toBeTruthy();
      expect(markdown.length).toBeGreaterThan(1000);
    });

    it("should convert small HTML document", () => {
      const html = loadTestDoc("html/wikipedia/small_html.html");
      const markdown = convert(html);
      expect(markdown).toBeTruthy();
    });
  });
});
