package io.github.goldziher.htmltomarkdown;

import java.lang.foreign.Arena;
import java.lang.foreign.MemorySegment;

/**
 * High-performance HTML to Markdown converter with Rust core.
 * <p>
 * This class provides a Java interface to the native html-to-markdown library
 * using Java's Foreign Function & Memory API (Panama FFI).
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
 * @since 2.7.2
 */
public final class HtmlToMarkdown {

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
            // Convert Java string to C string
            MemorySegment htmlSegment = HtmlToMarkdownFFI.toCString(arena, html);

            // Call native conversion function
            MemorySegment resultSegment = (MemorySegment) HtmlToMarkdownFFI.html_to_markdown_convert
                .invoke(htmlSegment);

            // Check for NULL return (indicates error)
            if (resultSegment == null || resultSegment.address() == 0) {
                String error = getLastError();
                throw new ConversionException(
                    error != null ? error : "Conversion failed with unknown error"
                );
            }

            try {
                // Convert C string to Java string
                return HtmlToMarkdownFFI.fromCString(resultSegment);
            } finally {
                // Free the native string
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
     * Get the last error message from a failed conversion.
     * <p>
     * Note: This is currently a placeholder. Full error handling will be added in a future version.
     *
     * @return the last error message, or null if no error occurred
     */
    private static String getLastError() {
        try {
            MemorySegment errorSegment = (MemorySegment) HtmlToMarkdownFFI.html_to_markdown_last_error.invoke();
            return HtmlToMarkdownFFI.fromCString(errorSegment);
        } catch (Throwable e) {
            return null;
        }
    }

    // Private constructor to prevent instantiation
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
