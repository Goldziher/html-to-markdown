import { readFile } from "node:fs/promises";
import type { Readable } from "node:stream";

import {
	convert as convertHtml,
	convertWithInlineImages as convertHtmlWithInlineImages,
	convertWithMetadata as convertHtmlWithMetadata,
	convertWithMetadataBuffer as convertHtmlWithMetadataBuffer,
	type JsConversionOptions,
	type JsHtmlExtraction,
	type JsInlineImageConfig,
	type JsMetadataConfig,
	type JsMetadataExtraction,
	type JsDocumentMetadata,
	type JsHeaderMetadata,
	type JsLinkMetadata,
	type JsImageMetadata,
	type JsStructuredData,
	type JsExtendedMetadata,
} from "html-to-markdown-node";

export * from "html-to-markdown-node";

/**
 * Check if metadata extraction functionality is available.
 *
 * @returns true if convertWithMetadata is available, false otherwise
 */
export function hasMetadataSupport(): boolean {
	try {
		return typeof convertHtmlWithMetadata === "function";
	} catch {
		return false;
	}
}

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

/**
 * Convert HTML to Markdown and extract comprehensive metadata.
 *
 * Extracts document metadata (title, description, language, etc.), headers,
 * links, images, and structured data (JSON-LD, Microdata, RDFa).
 *
 * @param html HTML content to convert
 * @param options Optional conversion configuration
 * @param metadataConfig Optional metadata extraction configuration
 * @returns Object with converted markdown and extracted metadata
 *
 * @example
 * ```ts
 * import { convertWithMetadata } from 'html-to-markdown';
 *
 * const html = `
 *   <html lang="en">
 *     <head>
 *       <title>My Article</title>
 *       <meta name="description" content="An interesting article">
 *     </head>
 *     <body>
 *       <h1>Main Title</h1>
 *       <p>Content with <a href="/page">link</a></p>
 *     </body>
 *   </html>
 * `;
 *
 * const { markdown, metadata } = await convertWithMetadata(html, undefined, {
 *   extractHeaders: true,
 *   extractLinks: true,
 *   extractImages: true,
 * });
 *
 * console.log(metadata.document.title);    // "My Article"
 * console.log(metadata.headers.length);    // 1
 * console.log(metadata.links.length);      // 1
 * ```
 */
export function convertWithMetadata(
	html: string,
	options?: JsConversionOptions | null | undefined,
	metadataConfig?: JsMetadataConfig | null | undefined,
): JsMetadataExtraction {
	return convertHtmlWithMetadata(html, options ?? undefined, metadataConfig ?? undefined);
}

/**
 * Convert HTML from Buffer/Uint8Array to Markdown with metadata extraction.
 *
 * Avoids creating intermediate JavaScript strings by accepting raw bytes.
 * Auto-detects UTF-8 encoding.
 */
export function convertWithMetadataBuffer(
	html: Buffer | Uint8Array,
	options?: JsConversionOptions | null | undefined,
	metadataConfig?: JsMetadataConfig | null | undefined,
): JsMetadataExtraction {
	return convertHtmlWithMetadataBuffer(html, options ?? undefined, metadataConfig ?? undefined);
}

/**
 * Convert the contents of an HTML file to Markdown with metadata extraction.
 */
export async function convertFileWithMetadata(
	filePath: string,
	options?: JsConversionOptions | null | undefined,
	metadataConfig?: JsMetadataConfig | null | undefined,
): Promise<JsMetadataExtraction> {
	const html = await readFile(filePath, "utf8");
	return convertWithMetadata(html, options ?? undefined, metadataConfig ?? undefined);
}

/**
 * Convert HTML streamed from stdin or another readable stream with metadata extraction.
 */
export async function convertStreamWithMetadata(
	stream: Readable | AsyncIterable<string | Buffer>,
	options?: JsConversionOptions | null | undefined,
	metadataConfig?: JsMetadataConfig | null | undefined,
): Promise<JsMetadataExtraction> {
	let html = "";

	for await (const chunk of stream as AsyncIterable<string | Buffer>) {
		html += typeof chunk === "string" ? chunk : chunk.toString("utf8");
	}

	return convertWithMetadata(html, options ?? undefined, metadataConfig ?? undefined);
}
