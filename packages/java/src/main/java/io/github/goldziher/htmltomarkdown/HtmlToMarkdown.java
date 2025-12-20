package io.github.goldziher.htmltomarkdown;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import io.github.goldziher.htmltomarkdown.metadata.DocumentMetadata;
import io.github.goldziher.htmltomarkdown.metadata.ExtendedMetadata;
import io.github.goldziher.htmltomarkdown.metadata.HeaderMetadata;
import io.github.goldziher.htmltomarkdown.metadata.ImageMetadata;
import io.github.goldziher.htmltomarkdown.metadata.ImageType;
import io.github.goldziher.htmltomarkdown.metadata.LinkMetadata;
import io.github.goldziher.htmltomarkdown.metadata.LinkType;
import io.github.goldziher.htmltomarkdown.metadata.MetadataExtraction;
import io.github.goldziher.htmltomarkdown.metadata.StructuredData;
import io.github.goldziher.htmltomarkdown.metadata.TextDirection;
import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/**
 * High-performance HTML to Markdown converter with Rust core.
 * <p>
 * This class provides a Java interface to the native html-to-markdown library
 * using Java's Foreign Function &amp; Memory API (Panama FFI).
 * <p>
 * <b>Example usage:</b>
 * <pre>{@code
 * String html = "<h1>Hello World</h1><p>This is a <strong>test</strong>.</p>";
 * String markdown = HtmlToMarkdown.convert(html);
 * System.out.println(markdown);
 * // Output:
 * // # Hello World
 * //
 * // This is a **test**.
 * }</pre>
 *
 * <p>
 * <b>Thread Safety:</b> This class is thread-safe. Multiple threads can safely
 * call {@link #convert(String)} concurrently.
 *
 * <p>
 * <b>Performance:</b> The underlying Rust implementation provides 60-80x higher
 * throughput compared to pure Java HTML-to-Markdown converters.
 *
 * @since 2.7.3
 */
public final class HtmlToMarkdown {
    private static final int DEFAULT_PROFILING_FREQUENCY = 1000;

    /**
     * Convert HTML to Markdown using default options.
     * <p>
     * This method uses CommonMark-compliant defaults:
     * <ul>
     *   <li>ATX-style headings ({@code # Heading})</li>
     *   <li>Two-space line breaks</li>
     *   <li>Cycling bullets for nested lists ({@code * + -})</li>
     *   <li>Minimal character escaping</li>
     * </ul>
     *
     * @param html the HTML string to convert
     * @return the converted Markdown string
     * @throws NullPointerException if html is null
     * @throws ConversionException if the conversion fails
     */
    public static String convert(String html) {
        if (html == null) {
            throw new NullPointerException("HTML cannot be null");
        }

        try (Arena arena = Arena.ofConfined()) {
            MemorySegment htmlSegment = HtmlToMarkdownFFI.toCString(arena, html);

            MemorySegment resultSegment = (MemorySegment) HtmlToMarkdownFFI.html_to_markdown_convert
                .invoke(htmlSegment);

            if (resultSegment == null || resultSegment.address() == 0) {
                String error = getLastError();
                throw new ConversionException(
                    error != null ? error : "Conversion failed with unknown error"
                );
            }

            try {
                return HtmlToMarkdownFFI.fromCString(resultSegment);
            } finally {
                HtmlToMarkdownFFI.html_to_markdown_free_string.invoke(resultSegment);
            }
        } catch (ConversionException e) {
            throw e;
        } catch (Throwable e) {
            throw new ConversionException("Failed to convert HTML to Markdown", e);
        }
    }

    /**
     * Get the version of the native html-to-markdown library.
     *
     * @return the library version string (e.g., "2.7.2")
     */
    public static String getVersion() {
        try {
            MemorySegment versionSegment = (MemorySegment) HtmlToMarkdownFFI.html_to_markdown_version.invoke();
            return HtmlToMarkdownFFI.fromCString(versionSegment);
        } catch (Throwable e) {
            throw new RuntimeException("Failed to get library version", e);
        }
    }

    /**
     * Start Rust-side profiling and write a flamegraph to the given output path.
     *
     * @param outputPath path to the flamegraph SVG to write
     * @param frequency sampling frequency in Hz (defaults to 1000 if <= 0)
     */
    public static void startProfiling(String outputPath, int frequency) {
        if (outputPath == null || outputPath.isBlank()) {
            throw new IllegalArgumentException("outputPath is required");
        }
        int freq = frequency > 0 ? frequency : DEFAULT_PROFILING_FREQUENCY;

        try (Arena arena = Arena.ofConfined()) {
            MemorySegment outputSegment = HtmlToMarkdownFFI.toCString(arena, outputPath);
            boolean ok = (boolean) HtmlToMarkdownFFI.html_to_markdown_profile_start
                .invoke(outputSegment, freq);
            if (!ok) {
                String error = getLastError();
                throw new ConversionException(
                    error != null ? error : "Profiling start failed"
                );
            }
        } catch (ConversionException e) {
            throw e;
        } catch (Throwable e) {
            throw new ConversionException("Failed to start profiling", e);
        }
    }

    /**
     * Stop Rust-side profiling and flush the flamegraph to disk.
     */
    public static void stopProfiling() {
        try {
            boolean ok = (boolean) HtmlToMarkdownFFI.html_to_markdown_profile_stop.invoke();
            if (!ok) {
                String error = getLastError();
                throw new ConversionException(
                    error != null ? error : "Profiling stop failed"
                );
            }
        } catch (ConversionException e) {
            throw e;
        } catch (Throwable e) {
            throw new ConversionException("Failed to stop profiling", e);
        }
    }

    /**
     * Convert HTML to Markdown with metadata extraction.
     * <p>
     * This method converts HTML to Markdown while extracting document metadata
     * such as titles, headers, links, images, and structured data.
     *
     * <p>
     * <b>Example usage:</b>
     * <pre>{@code
     * String html = "<html><head><title>Test</title></head><body><h1>Hello</h1><a href=\"https://example.com\">Link</a></body></html>";
     * MetadataExtraction result = HtmlToMarkdown.convertWithMetadata(html);
     * System.out.println("Markdown: " + result.getMarkdown());
     * System.out.println("Title: " + result.getDocumentMetadata().title());
     * System.out.println("Links: " + result.getMetadata().getLinkCount());
     * }</pre>
     *
     * @param html the HTML string to convert
     * @return a {@code MetadataExtraction} containing both markdown and metadata
     * @throws NullPointerException if html is null
     * @throws ConversionException if the conversion fails
     *
     * @since 2.13.0
     */
    public static MetadataExtraction convertWithMetadata(String html) {
        if (html == null) {
            throw new NullPointerException("HTML cannot be null");
        }

        try (Arena arena = Arena.ofConfined()) {
            MemorySegment htmlSegment = HtmlToMarkdownFFI.toCString(arena, html);

            MemorySegment metadataJsonOut = arena.allocate(
                java.lang.foreign.ValueLayout.ADDRESS
            );

            MemorySegment resultSegment = (MemorySegment) HtmlToMarkdownFFI.html_to_markdown_convert_with_metadata
                .invoke(htmlSegment, metadataJsonOut);

            if (resultSegment == null || resultSegment.address() == 0) {
                String error = getLastError();
                throw new ConversionException(
                    error != null ? error : "Conversion failed with unknown error"
                );
            }

            try {
                String markdown = HtmlToMarkdownFFI.fromCString(resultSegment);

                MemorySegment metadataJsonSegment = metadataJsonOut.getAtIndex(
                    java.lang.foreign.ValueLayout.ADDRESS,
                    0
                );

                ExtendedMetadata metadata;
                if (metadataJsonSegment != null && metadataJsonSegment.address() != 0) {
                    String metadataJson = HtmlToMarkdownFFI.fromCString(metadataJsonSegment);
                    metadata = parseMetadata(metadataJson);
                    HtmlToMarkdownFFI.html_to_markdown_free_string.invoke(metadataJsonSegment);
                } else {
                    metadata = ExtendedMetadata.empty();
                }

                return new MetadataExtraction(markdown, metadata);
            } finally {
                HtmlToMarkdownFFI.html_to_markdown_free_string.invoke(resultSegment);
            }
        } catch (ConversionException e) {
            throw e;
        } catch (Throwable e) {
            throw new ConversionException("Failed to convert HTML to Markdown with metadata", e);
        }
    }

    /**
     * Parse JSON metadata string into ExtendedMetadata.
     *
     * @param jsonStr the JSON metadata string
     * @return parsed {@code ExtendedMetadata}
     * @throws ConversionException if JSON parsing fails
     */
    private static ExtendedMetadata parseMetadata(String jsonStr) {
        try {
            ObjectMapper mapper = new ObjectMapper();
            JsonNode root = mapper.readTree(jsonStr);

            DocumentMetadata document = parseDocumentMetadata(mapper, root.get("document"));

            List<HeaderMetadata> headers = parseHeaders(root.get("headers"));

            List<LinkMetadata> links = parseLinks(root.get("links"));

            List<ImageMetadata> images = parseImages(root.get("images"));

            List<StructuredData> structuredData = parseStructuredData(root.get("structured_data"));

            return new ExtendedMetadata(document, headers, links, images, structuredData);
        } catch (Exception e) {
            throw new ConversionException("Failed to parse metadata JSON: " + e.getMessage(), e);
        }
    }

    /**
     * Parse document metadata from JSON node.
     */
    private static DocumentMetadata parseDocumentMetadata(ObjectMapper mapper, JsonNode node) {
        if (node == null) {
            return new DocumentMetadata(
                null, null, Collections.emptyList(), null, null, null, null, null,
                Collections.emptyMap(), Collections.emptyMap(), Collections.emptyMap()
            );
        }

        String title = node.has("title") && !node.get("title").isNull()
            ? node.get("title").asText() : null;
        String description = node.has("description") && !node.get("description").isNull()
            ? node.get("description").asText() : null;
        String author = node.has("author") && !node.get("author").isNull()
            ? node.get("author").asText() : null;
        String canonicalUrl = node.has("canonical_url") && !node.get("canonical_url").isNull()
            ? node.get("canonical_url").asText() : null;
        String baseHref = node.has("base_href") && !node.get("base_href").isNull() ? node.get("base_href").asText() : null;
        String language = node.has("language") && !node.get("language").isNull() ? node.get("language").asText() : null;
        TextDirection textDirection = null;
        if (node.has("text_direction") && !node.get("text_direction").isNull()) {
            String textDirectionStr = node.get("text_direction").asText();
            try {
                textDirection = TextDirection.parse(textDirectionStr);
            } catch (IllegalArgumentException e) {
                textDirection = null;
            }
        }

        List<String> keywords = new ArrayList<>();
        if (node.has("keywords") && node.get("keywords").isArray()) {
            node.get("keywords").forEach(kw -> keywords.add(kw.asText()));
        }

        Map<String, String> openGraph = new TreeMap<>();
        if (node.has("open_graph") && node.get("open_graph").isObject()) {
            node.get("open_graph").fields().forEachRemaining(e -> openGraph.put(e.getKey(), e.getValue().asText()));
        }

        Map<String, String> twitterCard = new TreeMap<>();
        if (node.has("twitter_card") && node.get("twitter_card").isObject()) {
            node.get("twitter_card").fields().forEachRemaining(e -> twitterCard.put(e.getKey(), e.getValue().asText()));
        }

        Map<String, String> metaTags = new TreeMap<>();
        if (node.has("meta_tags") && node.get("meta_tags").isObject()) {
            node.get("meta_tags").fields().forEachRemaining(e -> metaTags.put(e.getKey(), e.getValue().asText()));
        }

        return new DocumentMetadata(
            title, description, keywords, author, canonicalUrl, baseHref, language, textDirection,
            openGraph, twitterCard, metaTags
        );
    }

    /**
     * Parse headers from JSON node.
     */
    private static List<HeaderMetadata> parseHeaders(JsonNode node) {
        List<HeaderMetadata> headers = new ArrayList<>();
        if (node == null || !node.isArray()) {
            return headers;
        }

        node.forEach(h -> {
            int level = h.has("level") ? h.get("level").asInt() : 1;
            String text = h.has("text") ? h.get("text").asText() : "";
            String id = h.has("id") && !h.get("id").isNull() ? h.get("id").asText() : null;
            int depth = h.has("depth") ? h.get("depth").asInt() : 0;
            int htmlOffset = h.has("html_offset") ? h.get("html_offset").asInt() : 0;

            headers.add(new HeaderMetadata(level, text, id, depth, htmlOffset));
        });

        return headers;
    }

    /**
     * Parse links from JSON node.
     */
    private static List<LinkMetadata> parseLinks(JsonNode node) {
        List<LinkMetadata> links = new ArrayList<>();
        if (node == null || !node.isArray()) {
            return links;
        }

        node.forEach(l -> {
            String href = l.has("href") ? l.get("href").asText() : "";
            String text = l.has("text") ? l.get("text").asText() : "";
            String title = l.has("title") && !l.get("title").isNull() ? l.get("title").asText() : null;
            LinkType linkType = LinkType.OTHER;
            if (l.has("link_type")) {
                String linkTypeStr = l.get("link_type").asText();
                try {
                    linkType = LinkType.parse(linkTypeStr);
                } catch (IllegalArgumentException e) {
                    linkType = LinkType.OTHER;
                }
            }

            List<String> rel = new ArrayList<>();
            if (l.has("rel") && l.get("rel").isArray()) {
                l.get("rel").forEach(r -> rel.add(r.asText()));
            }

            Map<String, String> attributes = new HashMap<>();
            if (l.has("attributes") && l.get("attributes").isObject()) {
                l.get("attributes").fields().forEachRemaining(e -> attributes.put(e.getKey(), e.getValue().asText()));
            }

            links.add(new LinkMetadata(href, text, title, linkType, rel, attributes));
        });

        return links;
    }

    /**
     * Parse images from JSON node.
     */
    private static List<ImageMetadata> parseImages(JsonNode node) {
        List<ImageMetadata> images = new ArrayList<>();
        if (node == null || !node.isArray()) {
            return images;
        }

        node.forEach(img -> {
            String src = img.has("src") ? img.get("src").asText() : "";
            String alt = img.has("alt") && !img.get("alt").isNull() ? img.get("alt").asText() : null;
            String title = img.has("title") && !img.get("title").isNull() ? img.get("title").asText() : null;
            ImageType imageType = ImageType.RELATIVE;
            if (img.has("image_type")) {
                String imageTypeStr = img.get("image_type").asText();
                try {
                    imageType = ImageType.parse(imageTypeStr);
                } catch (IllegalArgumentException e) {
                    imageType = ImageType.RELATIVE;
                }
            }

            int[] dimensions = null;
            if (img.has("dimensions") && img.get("dimensions").isArray() && img.get("dimensions").size() == 2) {
                dimensions = new int[] {
                    img.get("dimensions").get(0).asInt(),
                    img.get("dimensions").get(1).asInt()
                };
            }

            Map<String, String> attributes = new HashMap<>();
            if (img.has("attributes") && img.get("attributes").isObject()) {
                img.get("attributes").fields().forEachRemaining(e -> attributes.put(e.getKey(), e.getValue().asText()));
            }

            images.add(new ImageMetadata(src, alt, title, dimensions, imageType, attributes));
        });

        return images;
    }

    /**
     * Parse structured data from JSON node.
     */
    private static List<StructuredData> parseStructuredData(JsonNode node) {
        List<StructuredData> structuredData = new ArrayList<>();
        if (node == null || !node.isArray()) {
            return structuredData;
        }

        node.forEach(sd -> {
            String dataType = sd.has("data_type") ? sd.get("data_type").asText() : "json_ld";
            String rawJson = sd.has("raw_json") ? sd.get("raw_json").asText() : "{}";
            String schemaType = sd.has("schema_type") && !sd.get("schema_type").isNull()
                ? sd.get("schema_type").asText() : null;

            structuredData.add(new StructuredData(dataType, rawJson, schemaType));
        });

        return structuredData;
    }

    /**
     * Get the last error message from a failed conversion.
     * <p>
     * If retrieving the error message fails, logs the exception and returns a descriptive error string.
     *
     * @return the last error message, or a descriptive error string if retrieval failed
     */
    private static String getLastError() {
        try {
            MemorySegment errorSegment = (MemorySegment) HtmlToMarkdownFFI.html_to_markdown_last_error.invoke();
            return HtmlToMarkdownFFI.fromCString(errorSegment);
        } catch (Throwable e) {
            System.err.println("Failed to retrieve FFI error message: " + e.getMessage());
            e.printStackTrace(System.err);
            return "FFI error retrieval failed: " + e.getClass().getSimpleName();
        }
    }

    private HtmlToMarkdown() {
        throw new UnsupportedOperationException("Utility class");
    }

    /**
     * Exception thrown when HTML-to-Markdown conversion fails.
     */
    public static class ConversionException extends RuntimeException {
        public ConversionException(String message) {
            super(message);
        }

        public ConversionException(String message, Throwable cause) {
            super(message, cause);
        }
    }
}
