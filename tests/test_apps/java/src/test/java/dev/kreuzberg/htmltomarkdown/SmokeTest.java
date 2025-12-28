package dev.kreuzberg.htmltomarkdown;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class SmokeTest {
    @Test
    void testPackageLoads() {
        assertNotNull(HtmlToMarkdown.class);
    }

    @Test
    void testBasicConversion() {
        String html = "<p>Hello World</p>";
        String result = HtmlToMarkdown.convert(html);
        assertTrue(result.contains("Hello World"));
    }

    @Test
    void testWithOptions() {
        String html = "<h1>Title</h1>";
        String result = HtmlToMarkdown.convert(html);
        assertTrue(result.startsWith("#"));
    }

    @Test
    void testEmptyInput() {
        String result = HtmlToMarkdown.convert("");
        assertEquals("", result);
    }
}
