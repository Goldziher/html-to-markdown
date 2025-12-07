import { rm, writeFile } from "node:fs/promises";
import { Readable } from "node:stream";
import { describe, expect, it } from "vitest";
import { convert, convertFile, convertStream, JsHeadingStyle } from "../src/index";

const FIXTURE_HTML = "<h1>Hello</h1><p>Markdown</p>";

describe("html-to-markdown (TypeScript package)", () => {
	it("converts inline HTML strings", () => {
		const markdown = convert(FIXTURE_HTML, { headingStyle: JsHeadingStyle.Atx });
		expect(markdown).toContain("# Hello");
		expect(markdown).toContain("Markdown");
	});

	it("converts files", async () => {
		const path = "tmp-test.html";
		await writeFile(path, FIXTURE_HTML, "utf8");
		try {
			const markdown = await convertFile(path);
			expect(markdown).toContain("Hello");
		} finally {
			await rm(path, { force: true });
		}
	});

	it("converts streams", async () => {
		const stream = Readable.from([FIXTURE_HTML]);
		const markdown = await convertStream(stream, { headingStyle: JsHeadingStyle.Atx });
		expect(markdown).toContain("# Hello");
	});
});
