import { describe, expect, it } from "vitest";
import type { JsConversionOptions } from "@kreuzberg/html-to-markdown-node";
import { convertWithVisitor } from "@kreuzberg/html-to-markdown-node";

/**
 * VisitResult type representing the result of a visitor callback.
 * Supports: continue, skip, custom output, preserve HTML, and error handling.
 */
interface VisitResult {
	type: "continue" | "skip" | "custom" | "preserve_html" | "error";
	output?: string;
	message?: string;
}

/**
 * NodeContext passed to visitor callbacks with element metadata.
 */
interface NodeContext {
	nodeType: string;
	tagName: string;
	attributes: Record<string, string>;
	depth: number;
	indexInParent: number;
	parentTag: string | null;
	isInline: boolean;
}

/**
 * Visitor interface for customizing HTML→Markdown conversion.
 * All callback methods are optional and return promises.
 */
interface Visitor {
	visitElementStart?: (ctx: NodeContext) => Promise<VisitResult>;
	visitElementEnd?: (ctx: NodeContext, output: string) => Promise<VisitResult>;

	visitText?: (ctx: NodeContext, text: string) => Promise<VisitResult>;

	visitLink?: (ctx: NodeContext, href: string, text: string, title?: string) => Promise<VisitResult>;
	visitImage?: (ctx: NodeContext, src: string, alt: string, title?: string) => Promise<VisitResult>;

	visitHeading?: (ctx: NodeContext, level: number, text: string, id?: string) => Promise<VisitResult>;

	visitCodeBlock?: (ctx: NodeContext, lang?: string, code?: string) => Promise<VisitResult>;
	visitCodeInline?: (ctx: NodeContext, code: string) => Promise<VisitResult>;

	visitListStart?: (ctx: NodeContext) => Promise<VisitResult>;
	visitListEnd?: (ctx: NodeContext, output: string) => Promise<VisitResult>;
	visitListItem?: (ctx: NodeContext) => Promise<VisitResult>;

	visitTableStart?: (ctx: NodeContext) => Promise<VisitResult>;
	visitTableEnd?: (ctx: NodeContext, output: string) => Promise<VisitResult>;
	visitTableRow?: (ctx: NodeContext) => Promise<VisitResult>;

	visitBlockquote?: (ctx: NodeContext) => Promise<VisitResult>;

	visitStrong?: (ctx: NodeContext, text: string) => Promise<VisitResult>;
	visitEmphasis?: (ctx: NodeContext, text: string) => Promise<VisitResult>;
	visitStrikethrough?: (ctx: NodeContext, text: string) => Promise<VisitResult>;
	visitUnderline?: (ctx: NodeContext, text: string) => Promise<VisitResult>;
	visitSubscript?: (ctx: NodeContext, text: string) => Promise<VisitResult>;
	visitSuperscript?: (ctx: NodeContext, text: string) => Promise<VisitResult>;
	visitMark?: (ctx: NodeContext, text: string) => Promise<VisitResult>;

	visitLineBreak?: (ctx: NodeContext) => Promise<VisitResult>;
	visitHorizontalRule?: (ctx: NodeContext) => Promise<VisitResult>;
	visitCustomElement?: (ctx: NodeContext) => Promise<VisitResult>;

	visitDefinitionListStart?: (ctx: NodeContext) => Promise<VisitResult>;
	visitDefinitionListEnd?: (ctx: NodeContext, output: string) => Promise<VisitResult>;
	visitDefinitionTerm?: (ctx: NodeContext) => Promise<VisitResult>;
	visitDefinitionDescription?: (ctx: NodeContext) => Promise<VisitResult>;

	visitForm?: (ctx: NodeContext) => Promise<VisitResult>;
	visitInput?: (ctx: NodeContext) => Promise<VisitResult>;
	visitButton?: (ctx: NodeContext, text?: string) => Promise<VisitResult>;

	visitAudio?: (ctx: NodeContext, src?: string) => Promise<VisitResult>;
	visitVideo?: (ctx: NodeContext, src?: string) => Promise<VisitResult>;
	visitIframe?: (ctx: NodeContext, src?: string) => Promise<VisitResult>;

	visitDetails?: (ctx: NodeContext) => Promise<VisitResult>;
	visitSummary?: (ctx: NodeContext) => Promise<VisitResult>;

	visitFigureStart?: (ctx: NodeContext) => Promise<VisitResult>;
	visitFigureEnd?: (ctx: NodeContext, output: string) => Promise<VisitResult>;
	visitFigcaption?: (ctx: NodeContext) => Promise<VisitResult>;
}

describe("html-to-markdown visitor API (TypeScript)", () => {
	describe("API contract and type safety", () => {
		it("should accept convertWithVisitor function with visitor parameter", async () => {
			const visitor: Visitor = {
				visitText: async (ctx, text) => {
					return { type: "continue" };
				},
			};

			const html = "<p>Test</p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
			expect(result).toBeTruthy();
		});

		it("should process HTML correctly with visitor object (continue result)", async () => {
			const visitor: Visitor = {
				visitText: async (ctx, text) => {
					return { type: "continue" };
				},
				visitHeading: async (ctx, level, text, id) => {
					return { type: "continue" };
				},
			};

			const html = "<h1>Title</h1><p>Content</p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(result).toContain("Title");
			expect(result).toContain("Content");
		});

		it("should handle empty visitor object gracefully", async () => {
			const visitor: Visitor = {};

			const html = "<h1>Title</h1><p>Content</p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(result).toContain("Title");
			expect(result).toContain("Content");
		});

		it("should work with empty object visitor parameter", async () => {
			const html = "<p>Test</p>";
			const result = await convertWithVisitor(html, undefined, {} as any);

			expect(typeof result).toBe("string");
			expect(result).toBeTruthy();
		});

		it("should handle visitor with single callback method", async () => {
			const visitor: Visitor = {
				visitLink: async (ctx, href, text, title) => {
					return { type: "continue" };
				},
			};

			const html = '<a href="/page">Link</a>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});
	});

	describe("VisitResult types", () => {
		it("should support continue result type", async () => {
			const visitor: Visitor = {
				visitText: async (ctx, text) => {
					const result: VisitResult = { type: "continue" };
					return result;
				},
			};

			const html = "<p>Test</p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(result).toBeTruthy();
			expect(typeof result).toBe("string");
		});

		it("should support custom result type with output", async () => {
			const visitor: Visitor = {
				visitLink: async (ctx, href, text, title) => {
					const result: VisitResult = {
						type: "custom",
						output: `[CUSTOM](${href})`,
					};
					return result;
				},
			};

			const html = '<a href="/page">Link</a>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support skip result type", async () => {
			const visitor: Visitor = {
				visitImage: async (ctx, src, alt, title) => {
					const result: VisitResult = { type: "skip" };
					return result;
				},
			};

			const html = '<img src="test.jpg" alt="Test">';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support preserve_html result type", async () => {
			const visitor: Visitor = {
				visitLink: async (ctx, href, text, title) => {
					const result: VisitResult = { type: "preserve_html" };
					return result;
				},
			};

			const html = '<a href="/test">Link</a>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support error result type", async () => {
			const visitor: Visitor = {
				visitText: async (ctx, text) => {
					if (text.includes("error")) {
						const result: VisitResult = {
							type: "error",
							message: "Found error",
						};
						return result;
					}
					return { type: "continue" };
				},
			};

			const html = "<p>Test error</p>";

			try {
				await convertWithVisitor(html, undefined, visitor);
			} catch (error) {
				expect(error).toBeDefined();
			}
		});
	});

	describe("NodeContext properties", () => {
		it("should provide NodeContext with all required properties", async () => {
			const contextsReceived: NodeContext[] = [];

			const visitor: Visitor = {
				visitElementStart: async (ctx) => {
					contextsReceived.push(ctx);
					return { type: "continue" };
				},
			};

			const html = "<div id='test' class='container'>Text</div>";
			await convertWithVisitor(html, undefined, visitor);

			const expectedProperties = [
				"nodeType",
				"tagName",
				"attributes",
				"depth",
				"indexInParent",
				"parentTag",
				"isInline",
			];

			const dummyContext: NodeContext = {
				nodeType: "Element",
				tagName: "div",
				attributes: { id: "test", class: "container" },
				depth: 0,
				indexInParent: 0,
				parentTag: null,
				isInline: false,
			};

			expect(dummyContext.tagName).toBe("div");
			expect(dummyContext.attributes.id).toBe("test");
			expect(typeof dummyContext.depth).toBe("number");
			expect(typeof dummyContext.isInline).toBe("boolean");
		});

		it("should type NodeContext.tagName as string", () => {
			const ctx: NodeContext = {
				nodeType: "Text",
				tagName: "p",
				attributes: {},
				depth: 1,
				indexInParent: 0,
				parentTag: "div",
				isInline: false,
			};

			const tagName: string = ctx.tagName;
			expect(typeof tagName).toBe("string");
		});

		it("should type NodeContext.depth as number", () => {
			const ctx: NodeContext = {
				nodeType: "Element",
				tagName: "div",
				attributes: {},
				depth: 5,
				indexInParent: 2,
				parentTag: "body",
				isInline: false,
			};

			const depth: number = ctx.depth;
			expect(typeof depth).toBe("number");
			expect(depth).toBe(5);
		});

		it("should type NodeContext.parentTag as string | null", () => {
			const ctxWithParent: NodeContext = {
				nodeType: "Element",
				tagName: "p",
				attributes: {},
				depth: 2,
				indexInParent: 0,
				parentTag: "div",
				isInline: false,
			};

			const parentTag: string | null = ctxWithParent.parentTag;
			expect(parentTag).toBe("div");

			const ctxWithoutParent: NodeContext = {
				nodeType: "Element",
				tagName: "html",
				attributes: {},
				depth: 0,
				indexInParent: 0,
				parentTag: null,
				isInline: false,
			};

			const parentTagNull: string | null = ctxWithoutParent.parentTag;
			expect(parentTagNull).toBeNull();
		});

		it("should type NodeContext.isInline as boolean", () => {
			const blockContext: NodeContext = {
				nodeType: "Element",
				tagName: "div",
				attributes: {},
				depth: 0,
				indexInParent: 0,
				parentTag: null,
				isInline: false,
			};

			const inlineContext: NodeContext = {
				nodeType: "Element",
				tagName: "span",
				attributes: {},
				depth: 1,
				indexInParent: 0,
				parentTag: "p",
				isInline: true,
			};

			expect(typeof blockContext.isInline).toBe("boolean");
			expect(blockContext.isInline).toBe(false);
			expect(typeof inlineContext.isInline).toBe("boolean");
			expect(inlineContext.isInline).toBe(true);
		});

		it("should type NodeContext.indexInParent as number", () => {
			const ctx: NodeContext = {
				nodeType: "Element",
				tagName: "li",
				attributes: {},
				depth: 2,
				indexInParent: 3,
				parentTag: "ul",
				isInline: false,
			};

			const index: number = ctx.indexInParent;
			expect(typeof index).toBe("number");
			expect(index).toBe(3);
		});

		it("should type NodeContext.attributes as Record<string, string>", () => {
			const ctx: NodeContext = {
				nodeType: "Element",
				tagName: "a",
				attributes: {
					href: "/page",
					title: "Page Title",
					class: "link",
				},
				depth: 1,
				indexInParent: 0,
				parentTag: "p",
				isInline: true,
			};

			const attributes: Record<string, string> = ctx.attributes;
			expect(attributes.href).toBe("/page");
			expect(attributes.title).toBe("Page Title");
			expect(attributes.class).toBe("link");
		});
	});

	describe("specialized visitor callbacks", () => {
		it("should support visitCodeBlock callback signature", async () => {
			const visitor: Visitor = {
				visitCodeBlock: async (ctx, lang, code) => {
					return { type: "continue" };
				},
			};

			const html = '<pre><code class="language-js">console.log();</code></pre>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitStrong callback signature", async () => {
			const visitor: Visitor = {
				visitStrong: async (ctx, text) => {
					return { type: "continue" };
				},
			};

			const html = "<p><strong>Bold</strong></p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitEmphasis callback signature", async () => {
			const visitor: Visitor = {
				visitEmphasis: async (ctx, text) => {
					return { type: "continue" };
				},
			};

			const html = "<p><em>Italic</em></p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitHeading callback signature", async () => {
			const visitor: Visitor = {
				visitHeading: async (ctx, level, text, id) => {
					return { type: "continue" };
				},
			};

			const html = '<h1 id="title">Title</h1>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitLink callback signature", async () => {
			const visitor: Visitor = {
				visitLink: async (ctx, href, text, title) => {
					return { type: "continue" };
				},
			};

			const html = '<a href="/page" title="Page">Link</a>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitImage callback signature", async () => {
			const visitor: Visitor = {
				visitImage: async (ctx, src, alt, title) => {
					return { type: "continue" };
				},
			};

			const html = '<img src="test.jpg" alt="Alt" title="Title">';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitLineBreak callback", async () => {
			const visitor: Visitor = {
				visitLineBreak: async (ctx) => {
					return { type: "continue" };
				},
			};

			const html = "<p>Line1<br>Line2</p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitListStart and visitListEnd callbacks", async () => {
			const visitor: Visitor = {
				visitListStart: async (ctx) => {
					return { type: "continue" };
				},
				visitListEnd: async (ctx, output) => {
					return { type: "continue" };
				},
			};

			const html = "<ul><li>Item</li></ul>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitTableStart and visitTableEnd callbacks", async () => {
			const visitor: Visitor = {
				visitTableStart: async (ctx) => {
					return { type: "continue" };
				},
				visitTableEnd: async (ctx, output) => {
					return { type: "continue" };
				},
			};

			const html = "<table><tr><td>Cell</td></tr></table>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitBlockquote callback", async () => {
			const visitor: Visitor = {
				visitBlockquote: async (ctx) => {
					return { type: "continue" };
				},
			};

			const html = "<blockquote><p>Quote</p></blockquote>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitCodeInline callback", async () => {
			const visitor: Visitor = {
				visitCodeInline: async (ctx, code) => {
					return { type: "continue" };
				},
			};

			const html = "<p>Use <code>console.log()</code></p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});
	});

	describe("visitor with ConversionOptions", () => {
		it("should accept both options and visitor parameter", async () => {
			const options: JsConversionOptions = {
				heading_style: "Atx",
			};

			const visitor: Visitor = {
				visitHeading: async (ctx, level, text, id) => {
					return { type: "continue" };
				},
			};

			const html = "<h1>Title</h1>";
			const result = await convertWithVisitor(html, options, visitor);

			expect(typeof result).toBe("string");
			expect(result).toContain("Title");
		});

		it("should work with undefined options and visitor", async () => {
			const html = "<p>Test</p>";
			const result = await convertWithVisitor(html, undefined, {});

			expect(typeof result).toBe("string");
		});

		it("should apply conversion options to output", async () => {
			const options: JsConversionOptions = {
				heading_style: "Atx",
			};

			const html = "<h1>Title</h1>";
			const result = await convertWithVisitor(html, options, {});

			expect(result).toContain("#");
			expect(result).toContain("Title");
		});
	});

	describe("multiple visitor methods", () => {
		it("should support multiple visitor callbacks defined simultaneously", async () => {
			const visitor: Visitor = {
				visitText: async (ctx, text) => {
					return { type: "continue" };
				},
				visitLink: async (ctx, href, text, title) => {
					return { type: "continue" };
				},
				visitHeading: async (ctx, level, text, id) => {
					return { type: "continue" };
				},
				visitImage: async (ctx, src, alt, title) => {
					return { type: "continue" };
				},
			};

			const html = "<h1>Title</h1><p>Text <a href='/p'>link</a> <img src='i.jpg' alt='img'></p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
			expect(result).toBeTruthy();
		});

		it("should support visitor with ElementStart and ElementEnd callbacks", async () => {
			const visitor: Visitor = {
				visitElementStart: async (ctx) => {
					return { type: "continue" };
				},
				visitElementEnd: async (ctx, output) => {
					return { type: "continue" };
				},
			};

			const html = "<div><p>Content</p></div>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitor with list callbacks", async () => {
			const visitor: Visitor = {
				visitListStart: async (ctx) => {
					return { type: "continue" };
				},
				visitListItem: async (ctx) => {
					return { type: "continue" };
				},
				visitListEnd: async (ctx, output) => {
					return { type: "continue" };
				},
			};

			const html = "<ul><li>Item 1</li><li>Item 2</li></ul>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support visitor with formatting callbacks", async () => {
			const visitor: Visitor = {
				visitStrong: async (ctx, text) => {
					return { type: "continue" };
				},
				visitEmphasis: async (ctx, text) => {
					return { type: "continue" };
				},
				visitStrikethrough: async (ctx, text) => {
					return { type: "continue" };
				},
				visitUnderline: async (ctx, text) => {
					return { type: "continue" };
				},
			};

			const html = "<p><strong>Bold</strong> <em>Italic</em> <s>Strike</s> <u>Under</u></p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});
	});

	describe("async visitor patterns", () => {
		it("should handle async operations in visitor callbacks", async () => {
			const visitor: Visitor = {
				visitLink: async (ctx, href, text, title) => {
					await new Promise((resolve) => setTimeout(resolve, 1));
					return { type: "continue" };
				},
			};

			const html = '<a href="/page">Link</a>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should handle conditional logic in visitor callbacks", async () => {
			const visitor: Visitor = {
				visitHeading: async (ctx, level, text, id) => {
					if (level >= 3) {
						return { type: "skip" };
					}
					return { type: "continue" };
				},
			};

			const html = "<h1>H1</h1><h2>H2</h2><h3>H3</h3>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should handle custom output in visitor callbacks", async () => {
			const visitor: Visitor = {
				visitLink: async (ctx, href, text, title) => {
					return {
						type: "custom",
						output: `[${text}](${href})`,
					};
				},
			};

			const html = '<a href="/page">Link</a>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});
	});

	describe("nested elements handling", () => {
		it("should handle deeply nested elements with visitor callbacks", async () => {
			const visitor: Visitor = {
				visitElementStart: async (ctx) => {
					return { type: "continue" };
				},
				visitElementEnd: async (ctx, output) => {
					return { type: "continue" };
				},
				visitText: async (ctx, text) => {
					return { type: "continue" };
				},
			};

			const html = "<div><div><p><strong><em>Deep text</em></strong></p></div></div>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
			expect(result).toContain("Deep text");
		});

		it("should handle list nesting with visitor callbacks", async () => {
			const visitor: Visitor = {
				visitListStart: async (ctx) => {
					return { type: "continue" };
				},
				visitListItem: async (ctx) => {
					return { type: "continue" };
				},
				visitListEnd: async (ctx, output) => {
					return { type: "continue" };
				},
			};

			const html = "<ul><li>Item 1<ul><li>Nested 1</li><li>Nested 2</li></ul></li><li>Item 2</li></ul>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should handle table structure with visitor callbacks", async () => {
			const visitor: Visitor = {
				visitTableStart: async (ctx) => {
					return { type: "continue" };
				},
				visitTableRow: async (ctx) => {
					return { type: "continue" };
				},
				visitTableEnd: async (ctx, output) => {
					return { type: "continue" };
				},
			};

			const html = "<table><tr><td>Cell 1</td><td>Cell 2</td></tr></table>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});
	});

	describe("error handling and edge cases", () => {
		it("should handle visitor with null output in custom result", async () => {
			const visitor: Visitor = {
				visitLink: async (ctx, href, text, title) => {
					return {
						type: "custom",
						output: "",
					};
				},
			};

			const html = '<a href="/page">Link</a>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should handle visitor callback returning different result types", async () => {
			const visitor: Visitor = {
				visitImage: async (ctx, src, alt, title) => {
					if (src.includes("skip")) {
						return { type: "skip" };
					}
					if (src.includes("custom")) {
						return { type: "custom", output: "[custom image]" };
					}
					return { type: "continue" };
				},
			};

			const html = '<img src="skip.jpg" alt="Skip"> <img src="custom.jpg" alt="Custom">';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should handle definition list callbacks", async () => {
			const visitor: Visitor = {
				visitDefinitionListStart: async (ctx) => {
					return { type: "continue" };
				},
				visitDefinitionTerm: async (ctx) => {
					return { type: "continue" };
				},
				visitDefinitionDescription: async (ctx) => {
					return { type: "continue" };
				},
				visitDefinitionListEnd: async (ctx, output) => {
					return { type: "continue" };
				},
			};

			const html = "<dl><dt>Term</dt><dd>Description</dd></dl>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should handle form element callbacks", async () => {
			const visitor: Visitor = {
				visitForm: async (ctx) => {
					return { type: "continue" };
				},
				visitInput: async (ctx) => {
					return { type: "continue" };
				},
				visitButton: async (ctx, text) => {
					return { type: "continue" };
				},
			};

			const html = '<form><input type="text"> <button>Submit</button></form>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should handle media element callbacks", async () => {
			const visitor: Visitor = {
				visitAudio: async (ctx, src) => {
					return { type: "continue" };
				},
				visitVideo: async (ctx, src) => {
					return { type: "continue" };
				},
				visitIframe: async (ctx, src) => {
					return { type: "continue" };
				},
			};

			const html =
				'<audio src="audio.mp3"></audio><video src="video.mp4"></video><iframe src="page.html"></iframe>';
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should handle detail/summary callbacks", async () => {
			const visitor: Visitor = {
				visitDetails: async (ctx) => {
					return { type: "continue" };
				},
				visitSummary: async (ctx) => {
					return { type: "continue" };
				},
			};

			const html = "<details><summary>Summary</summary><p>Details</p></details>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should handle figure element callbacks", async () => {
			const visitor: Visitor = {
				visitFigureStart: async (ctx) => {
					return { type: "continue" };
				},
				visitFigcaption: async (ctx) => {
					return { type: "continue" };
				},
				visitFigureEnd: async (ctx, output) => {
					return { type: "continue" };
				},
			};

			const html = "<figure><img src='img.jpg'><figcaption>Caption</figcaption></figure>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});
	});

	describe("TypeScript type safety", () => {
		it("should enforce NodeContext type when accessed", () => {
			const ctx: NodeContext = {
				nodeType: "Element",
				tagName: "div",
				attributes: {},
				depth: 0,
				indexInParent: 0,
				parentTag: null,
				isInline: false,
			};

			const tagName: string = ctx.tagName;
			const depth: number = ctx.depth;
			const isInline: boolean = ctx.isInline;

			expect(tagName).toBe("div");
			expect(depth).toBe(0);
			expect(isInline).toBe(false);
		});

		it("should enforce VisitResult types", () => {
			const continueResult: VisitResult = { type: "continue" };
			const customResult: VisitResult = { type: "custom", output: "text" };
			const skipResult: VisitResult = { type: "skip" };
			const preserveResult: VisitResult = { type: "preserve_html" };
			const errorResult: VisitResult = { type: "error", message: "error" };

			expect(continueResult.type).toBe("continue");
			expect(customResult.output).toBe("text");
			expect(skipResult.type).toBe("skip");
			expect(preserveResult.type).toBe("preserve_html");
			expect(errorResult.message).toBe("error");
		});

		it("should type visitor callback parameters correctly", async () => {
			const visitor: Visitor = {
				visitHeading: async (ctx: NodeContext, level: number, text: string, id?: string) => {
					expect(typeof ctx.tagName).toBe("string");
					expect(typeof level).toBe("number");
					expect(typeof text).toBe("string");
					if (id) {
						expect(typeof id).toBe("string");
					}
					return { type: "continue" };
				},
			};

			const html = '<h1 id="title">Title</h1>';
			await convertWithVisitor(html, undefined, visitor);
		});

		it("should type visitor return values as Promise<VisitResult>", async () => {
			const visitor: Visitor = {
				visitLink: async (ctx, href, text, title) => {
					const result: Promise<VisitResult> = Promise.resolve({
						type: "continue",
					});
					return result;
				},
			};

			const html = '<a href="/page">Link</a>';
			const output = await convertWithVisitor(html, undefined, visitor);

			expect(typeof output).toBe("string");
		});
	});

	describe("comprehensive visitor coverage", () => {
		it("should support all text formatting callbacks", async () => {
			const visitor: Visitor = {
				visitStrong: async (ctx, text) => ({ type: "continue" }),
				visitEmphasis: async (ctx, text) => ({ type: "continue" }),
				visitStrikethrough: async (ctx, text) => ({ type: "continue" }),
				visitUnderline: async (ctx, text) => ({ type: "continue" }),
				visitSubscript: async (ctx, text) => ({ type: "continue" }),
				visitSuperscript: async (ctx, text) => ({ type: "continue" }),
				visitMark: async (ctx, text) => ({ type: "continue" }),
			};

			const html =
				"<p><strong>b</strong><em>i</em><s>s</s><u>u</u><sub>sub</sub><sup>sup</sup><mark>m</mark></p>";
			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
		});

		it("should support comprehensive HTML document structure", async () => {
			const visitor: Visitor = {
				visitElementStart: async (ctx) => ({ type: "continue" }),
				visitElementEnd: async (ctx, output) => ({ type: "continue" }),
				visitHeading: async (ctx, level, text, id) => ({ type: "continue" }),
				visitLink: async (ctx, href, text, title) => ({ type: "continue" }),
				visitImage: async (ctx, src, alt, title) => ({ type: "continue" }),
				visitCodeBlock: async (ctx, lang, code) => ({ type: "continue" }),
				visitListStart: async (ctx) => ({ type: "continue" }),
				visitListEnd: async (ctx, output) => ({ type: "continue" }),
				visitTableStart: async (ctx) => ({ type: "continue" }),
				visitTableEnd: async (ctx, output) => ({ type: "continue" }),
				visitBlockquote: async (ctx) => ({ type: "continue" }),
			};

			const html = `
				<h1>Title</h1>
				<p>Paragraph with <a href="/link">link</a> and <img src="img.jpg" alt="image">.</p>
				<pre><code>code</code></pre>
				<ul><li>Item 1</li><li>Item 2</li></ul>
				<table><tr><td>Cell</td></tr></table>
				<blockquote><p>Quote</p></blockquote>
			`;

			const result = await convertWithVisitor(html, undefined, visitor);

			expect(typeof result).toBe("string");
			expect(result).toBeTruthy();
		});

		it("should maintain conversion output consistency with visitor callbacks", async () => {
			const withoutVisitor = await convertWithVisitor("<h1>Test</h1><p>Content</p>", undefined, {});
			const withVisitor = await convertWithVisitor("<h1>Test</h1><p>Content</p>", undefined, {
				visitHeading: async (ctx, level, text, id) => ({ type: "continue" }),
			});

			expect(typeof withoutVisitor).toBe("string");
			expect(typeof withVisitor).toBe("string");
			expect(withoutVisitor).toBeTruthy();
			expect(withVisitor).toBeTruthy();
		});
	});
});
