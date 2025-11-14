package io.github.goldziher.htmltomarkdown;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import static org.junit.jupiter.api.Assertions.*;

/**
 * Test suite for HtmlToMarkdown Java Panama FFI bindings.
 */
class HtmlToMarkdownTest {

    /** Number of sections for large document test. */
    private static final int LARGE_DOC_SECTIONS = 100;

    /** Minimum length for large document output. */
    private static final int LARGE_DOC_MIN_LENGTH = 1000;

    @Test
    @DisplayName("Basic heading conversion")
    void testBasicHeading() {
        String html = "<h1>Hello World</h1>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("# Hello World"),
            "Expected ATX-style heading, got: " + markdown);
    }

    @Test
    @DisplayName("Paragraph with strong text")
    void testParagraphWithStrong() {
        String html = "<p>This is a <strong>test</strong>.</p>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("**test**"),
            "Expected strong text to be converted to **text**, got: " + markdown);
    }

    @Test
    @DisplayName("Unordered list conversion")
    void testUnorderedList() {
        String html = "<ul><li>Item 1</li><li>Item 2</li><li>Item 3</li></ul>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("Item 1")
                   && markdown.contains("Item 2")
                   && markdown.contains("Item 3"),
            "Expected list items to be present, got: " + markdown);
    }

    @Test
    @DisplayName("Ordered list conversion")
    void testOrderedList() {
        String html = "<ol><li>First</li><li>Second</li><li>Third</li></ol>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("First")
                   && markdown.contains("Second")
                   && markdown.contains("Third"),
            "Expected ordered list items, got: " + markdown);
    }

    @Test
    @DisplayName("Link conversion")
    void testLink() {
        String html = "<a href=\"https://example.com\">Example</a>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("[Example]")
                   && markdown.contains("(https://example.com)"),
            "Expected markdown link format, got: " + markdown);
    }

    @Test
    @DisplayName("Code block conversion")
    void testCodeBlock() {
        String html = "<pre><code>"
            + "function test() { return true; }"
            + "</code></pre>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("function test()"),
            "Expected code block content, got: " + markdown);
    }

    @Test
    @DisplayName("Inline code conversion")
    void testInlineCode() {
        String html = "<p>Use the <code>convert()</code> function</p>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("`convert()`"),
            "Expected inline code in backticks, got: " + markdown);
    }

    @Test
    @DisplayName("Emphasis (em) conversion")
    void testEmphasis() {
        String html = "<p>This is <em>important</em>.</p>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("*important*")
                   || markdown.contains("_important_"),
            "Expected emphasis to be converted, got: " + markdown);
    }

    @Test
    @DisplayName("Blockquote conversion")
    void testBlockquote() {
        String html = "<blockquote>This is a quote</blockquote>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("> ")
                   && markdown.contains("This is a quote"),
            "Expected blockquote format, got: " + markdown);
    }

    @Test
    @DisplayName("Complex nested HTML")
    void testComplexNested() {
        String html = "<div><h2>Section</h2>"
            + "<p>Text with <strong>bold</strong> "
            + "and <em>italic</em>.</p>"
            + "<ul><li>Item 1</li><li>Item 2</li></ul></div>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("## Section")
                   && markdown.contains("**bold**")
                   && (markdown.contains("*italic*")
                       || markdown.contains("_italic_"))
                   && markdown.contains("Item 1"),
            "Expected all elements, got: " + markdown);
    }

    @Test
    @DisplayName("Empty string handling")
    void testEmptyString() {
        String html = "";
        String markdown = HtmlToMarkdown.convert(html);

        assertNotNull(markdown, "Result should not be null");
    }

    @Test
    @DisplayName("Null input throws exception")
    void testNullInput() {
        assertThrows(NullPointerException.class, () -> {
            HtmlToMarkdown.convert(null);
        }, "Should throw NullPointerException for null input");
    }

    @Test
    @DisplayName("Plain text (no HTML tags)")
    void testPlainText() {
        String html = "Just plain text";
        String markdown = HtmlToMarkdown.convert(html);

        assertEquals("Just plain text", markdown.trim(),
            "Plain text should pass through unchanged");
    }

    @Test
    @DisplayName("Table conversion")
    void testTable() {
        String html = "<table><tr><th>Name</th><th>Age</th></tr>"
            + "<tr><td>Alice</td><td>30</td></tr>"
            + "<tr><td>Bob</td><td>25</td></tr></table>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("Name")
                   && markdown.contains("Age")
                   && markdown.contains("Alice")
                   && markdown.contains("Bob"),
            "Expected table content, got: " + markdown);
    }

    @Test
    @DisplayName("Image conversion")
    void testImage() {
        String html = "<img src=\"image.png\" alt=\"Test Image\">";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("![Test Image]")
                   && markdown.contains("(image.png)"),
            "Expected markdown image format, got: " + markdown);
    }

    @Test
    @DisplayName("Horizontal rule conversion")
    void testHorizontalRule() {
        String html = "<p>Before</p><hr><p>After</p>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("Before")
                   && markdown.contains("After")
                   && (markdown.contains("---")
                       || markdown.contains("* * *")
                       || markdown.contains("___")),
            "Expected horizontal rule, got: " + markdown);
    }

    @Test
    @DisplayName("Get library version")
    void testGetVersion() {
        String version = HtmlToMarkdown.getVersion();

        assertNotNull(version, "Version should not be null");
        assertFalse(version.isEmpty(),
            "Version should not be empty");
        assertTrue(version.matches("\\d+\\.\\d+\\.\\d+"),
            "Version should match semver (x.y.z), got: " + version);
    }

    @Test
    @DisplayName("Multiple conversions in sequence")
    void testMultipleConversions() {
        String html1 = "<h1>First</h1>";
        String html2 = "<h2>Second</h2>";
        String html3 = "<h3>Third</h3>";

        String md1 = HtmlToMarkdown.convert(html1);
        String md2 = HtmlToMarkdown.convert(html2);
        String md3 = HtmlToMarkdown.convert(html3);

        assertTrue(md1.contains("# First"), "First conversion failed");
        assertTrue(md2.contains("## Second"), "Second conversion failed");
        assertTrue(md3.contains("### Third"), "Third conversion failed");
    }

    @Test
    @DisplayName("Large HTML document")
    void testLargeDocument() {
        StringBuilder html = new StringBuilder();
        html.append("<article>");
        for (int i = 1; i <= LARGE_DOC_SECTIONS; i++) {
            html.append("<h2>Section ").append(i).append("</h2>");
            html.append("<p>This is paragraph ").append(i)
                .append(" with <strong>bold</strong> text.</p>");
            html.append("<ul><li>Item A</li><li>Item B</li></ul>");
        }
        html.append("</article>");

        String markdown = HtmlToMarkdown.convert(html.toString());

        assertNotNull(markdown, "Markdown should not be null");
        assertTrue(markdown.contains("## Section 1"),
            "Should contain first section");
        assertTrue(markdown.contains("## Section 100"),
            "Should contain last section");
        assertTrue(markdown.length() > LARGE_DOC_MIN_LENGTH,
            "Markdown should be substantial in length");
    }

    @Test
    @DisplayName("Special characters in text")
    void testSpecialCharacters() {
        String html =
            "<p>Characters: &lt; &gt; &amp; &quot; &#39;</p>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("<")
                   || markdown.contains("&lt;"),
            "Expected special chars, got: " + markdown);
    }

    @Test
    @DisplayName("Nested lists")
    void testNestedLists() {
        String html = "<ul><li>Level 1<ul><li>Level 2"
            + "<ul><li>Level 3</li></ul></li></ul></li></ul>";
        String markdown = HtmlToMarkdown.convert(html);

        assertTrue(markdown.contains("Level 1")
                   && markdown.contains("Level 2")
                   && markdown.contains("Level 3"),
            "Expected nested list items, got: " + markdown);
    }
}
