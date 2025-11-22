import { readFile } from "node:fs/promises";
import type { Readable } from "node:stream";

import {
	convert as convertHtml,
	convertWithInlineImages as convertHtmlWithInlineImages,
	type JsConversionOptions,
	type JsHtmlExtraction,
	type JsInlineImageConfig,
} from "html-to-markdown-node";

export * from "html-to-markdown-node";

/**
 * Convert the contents of an HTML file to Markdown.
 */
export async function convertFile(filePath: string, options?: JsConversionOptions | null | undefined): Promise<string> {
	const html = await readFile(filePath, "utf8");
	return convertHtml(html, options ?? undefined);
}

/**
 * Convert an HTML file and collect inline images.
 */
export async function convertFileWithInlineImages(
	filePath: string,
	options?: JsConversionOptions | null | undefined,
	imageConfig?: JsInlineImageConfig | null | undefined,
): Promise<JsHtmlExtraction> {
	const html = await readFile(filePath, "utf8");
	return convertHtmlWithInlineImages(html, options ?? undefined, imageConfig ?? undefined);
}

/**
 * Convert HTML streamed from stdin or another readable stream.
 */
export async function convertStream(
	stream: Readable | AsyncIterable<string | Buffer>,
	options?: JsConversionOptions | null | undefined,
): Promise<string> {
	let html = "";

	for await (const chunk of stream as AsyncIterable<string | Buffer>) {
		html += typeof chunk === "string" ? chunk : chunk.toString("utf8");
	}

	return convertHtml(html, options ?? undefined);
}

/**
 * Convert HTML from a stream and collect inline images.
 */
export async function convertStreamWithInlineImages(
	stream: Readable | AsyncIterable<string | Buffer>,
	options?: JsConversionOptions | null | undefined,
	imageConfig?: JsInlineImageConfig | null | undefined,
): Promise<JsHtmlExtraction> {
	let html = "";

	for await (const chunk of stream as AsyncIterable<string | Buffer>) {
		html += typeof chunk === "string" ? chunk : chunk.toString("utf8");
	}

	return convertHtmlWithInlineImages(html, options ?? undefined, imageConfig ?? undefined);
}
