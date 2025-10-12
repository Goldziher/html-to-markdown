import { describe, it, expect, beforeAll } from "vitest";
import {
  convert,
  convertSync,
  convertWithInlineImages,
  initializeSync,
  getBackend,
  createConverter,
  HeadingStyle,
  CodeBlockStyle,
  ListIndentType,
  type ConversionOptions,
} from "./index.js";

describe("html-to-markdown", () => {
  beforeAll(() => {
    initializeSync();
  });

  describe("Basic Conversion", () => {
    it("should convert simple HTML to markdown", async () => {
      const html = "<h1>Hello World</h1>";
      const markdown = await convert(html);
      expect(markdown).toContain("Hello World");
    });

    it("should convert with options", async () => {
      const html = "<h1>Test</h1>";
      const options: ConversionOptions = {
        headingStyle: HeadingStyle.Atx,
      };
      const markdown = await convert(html, options);
      expect(markdown).toMatch(/^#\s+Test/);
    });

    it("should handle complex HTML", async () => {
      const html = `
        <h1>Title</h1>
        <p>Paragraph with <strong>bold</strong> and <em>italic</em>.</p>
        <ul>
          <li>Item 1</li>
          <li>Item 2</li>
        </ul>
      `;
      const markdown = await convert(html);
      expect(markdown).toContain("Title");
      expect(markdown).toMatch(/\*\*bold\*\*/);
      expect(markdown).toMatch(/\*italic\*/);
      expect(markdown).toMatch(/-\s+Item 1/);
    });
  });

  describe("Synchronous Conversion", () => {
    it("should convert synchronously after initialization", () => {
      const html = "<h1>Sync Test</h1>";
      const markdown = convertSync(html);
      expect(markdown).toContain("Sync Test");
    });

    it("should handle options synchronously", () => {
      const html = "<code>inline code</code>";
      const markdown = convertSync(html);
      expect(markdown).toMatch(/`inline code`/);
    });
  });

  describe("Options", () => {
    it("should respect heading style", async () => {
      const html = "<h1>Test</h1>";

      const atx = await convert(html, { headingStyle: HeadingStyle.Atx });
      expect(atx).toMatch(/^#\s+Test/);

      const underlined = await convert(html, {
        headingStyle: HeadingStyle.Underlined,
      });
      expect(underlined).toMatch(/Test\n={4,}/);
    });

    it("should respect code block style", async () => {
      const html = "<pre><code>function test() {}</code></pre>";

      const backticks = await convert(html, {
        codeBlockStyle: CodeBlockStyle.Backticks,
      });
      expect(backticks).toMatch(/```/);
    });

    it("should handle list indentation", async () => {
      const html = "<ul><li>Item</li></ul>";

      const spaces = await convert(html, {
        listIndentType: ListIndentType.Spaces,
        listIndentWidth: 4,
      });
      expect(spaces).toContain("Item");
    });

    it("should handle text wrapping", async () => {
      const longText = "a".repeat(200);
      const html = `<p>${longText}</p>`;

      const wrapped = await convert(html, {
        wrap: true,
        wrapWidth: 80,
      });

      const lines = wrapped.split("\n");
      const hasShortLines = lines.some((line) => line.length > 0 && line.length <= 85);
      expect(hasShortLines).toBe(true);
    });
  });

  describe("Inline Images", () => {
    it("should extract inline images", async () => {
      // Simple 1x1 red PNG in base64
      const png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==";
      const html = `<img src="data:image/png;base64,${png}" alt="Red Pixel">`;

      const result = await convertWithInlineImages(html, undefined, {
        maxDecodedSizeBytes: 1024,
        inferDimensions: true,
      });

      expect(result.markdown).toContain("Red Pixel");
      expect(result.inlineImages).toHaveLength(1);
      expect(result.inlineImages[0].format).toBe("png");
      expect(result.inlineImages[0].data).toBeInstanceOf(Buffer);
      expect(result.warnings).toHaveLength(0);
    });

    it("should handle invalid base64", async () => {
      const html = '<img src="data:image/png;base64,invalid!!!" alt="Broken">';

      const result = await convertWithInlineImages(html);

      expect(result.markdown).toContain("Broken");
      expect(result.inlineImages).toHaveLength(0);
      expect(result.warnings.length).toBeGreaterThan(0);
    });

    it("should extract SVG elements", async () => {
      const html = '<svg><circle cx="10" cy="10" r="5"/></svg>';

      const result = await convertWithInlineImages(html, undefined, {
        captureSvg: true,
      });

      const hasSvg = result.inlineImages.some((img) => img.format === "svg");
      expect(hasSvg).toBe(true);
    });
  });

  describe("Backend Detection", () => {
    it("should report backend type", () => {
      const backend = getBackend();
      expect(backend).toMatch(/^(native|wasm)$/);
    });
  });

  describe("Custom Converter Instance", () => {
    it("should create independent converter", async () => {
      const converter = createConverter();
      const markdown = await converter.convert("<h1>Custom</h1>");
      expect(markdown).toContain("Custom");
    });

    it("should check backend after conversion", async () => {
      const converter = createConverter();
      await converter.convert("<p>test</p>");
      const backend = converter.getBackend();
      expect(backend).toMatch(/^(native|wasm)$/);
    });
  });

  describe("Edge Cases", () => {
    it("should handle empty HTML", async () => {
      const markdown = await convert("");
      expect(markdown).toBe("");
    });

    it("should handle HTML with only whitespace", async () => {
      const markdown = await convert("   \n   \t   ");
      expect(markdown.trim()).toBe("");
    });

    it("should handle malformed HTML gracefully", async () => {
      const html = "<h1>Unclosed tag<p>Paragraph";
      const markdown = await convert(html);
      expect(markdown).toContain("Unclosed tag");
      expect(markdown).toContain("Paragraph");
    });

    it("should handle special characters", async () => {
      const html = "<p>Test & &lt;special&gt; characters</p>";
      const markdown = await convert(html);
      expect(markdown).toContain("&");
      expect(markdown).toContain("<special>");
    });

    it("should handle unicode", async () => {
      const html = "<p>Hello 世界 🌍</p>";
      const markdown = await convert(html);
      expect(markdown).toContain("世界");
      expect(markdown).toContain("🌍");
    });
  });

  describe("Preprocessing", () => {
    it("should remove navigation elements", async () => {
      const html = `
        <nav><a href="#">Menu</a></nav>
        <main><h1>Content</h1></main>
      `;

      const markdown = await convert(html, {
        preprocessing: {
          enabled: true,
          removeNavigation: true,
        },
      });

      expect(markdown).toContain("Content");
      expect(markdown).not.toContain("Menu");
    });

    it("should remove form elements", async () => {
      const html = `
        <h1>Title</h1>
        <form><input type="text"></form>
      `;

      const markdown = await convert(html, {
        preprocessing: {
          enabled: true,
          removeForms: true,
        },
      });

      expect(markdown).toContain("Title");
      expect(markdown).not.toContain("input");
    });
  });
});
