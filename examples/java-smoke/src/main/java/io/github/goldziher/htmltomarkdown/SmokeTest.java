package io.github.goldziher.htmltomarkdown;

/**
 * Smoke test for the html-to-markdown Java bindings.
 */
public final class SmokeTest {

    private SmokeTest() {
        // Prevent instantiation
    }

    /**
     * Main entry point for the smoke test.
     *
     * @param args command line arguments (unused)
     */
    public static void main(String[] args) {
        String html = """
            <h1>Java Smoke Test</h1>
            <p>Exercises the packaged Panama FFI bindings.</p>
            """;

        String markdown = HtmlToMarkdown.convert(html);

        if (!markdown.contains("# Java Smoke Test")) {
            System.err.println("html-to-markdown did not return the expected heading");
            System.exit(1);
        }

        System.out.println("✓ html-to-markdown (Java) produced markdown");
        System.out.println("---");
        System.out.println(markdown);
    }
}
