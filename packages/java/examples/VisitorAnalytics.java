package dev.kreuzberg.examples;

import dev.kreuzberg.htmltomarkdown.HtmlToMarkdown;
import dev.kreuzberg.htmltomarkdown.visitor.Visitor;
import dev.kreuzberg.htmltomarkdown.visitor.VisitResult;
import dev.kreuzberg.htmltomarkdown.visitor.NodeContext;

import java.util.HashMap;
import java.util.Map;
import java.util.TreeMap;

/**
 * Analytics visitor example.
 *
 * Demonstrates extracting statistics and metadata during HTML-to-Markdown
 * conversion without modifying the output.
 *
 * @since 2.17.0
 */
public class VisitorAnalytics {

    /**
     * Visitor that collects document statistics.
     */
    static class DocumentAnalyticsVisitor implements Visitor {
        /** Position of protocol separator in URL. */
        private static final int PROTOCOL_SEPARATOR_LENGTH = 3;
        /** Heading level 3 (h3). */
        private static final int HEADING_LEVEL_3 = 3;

        int headingCount = 0;
        int linkCount = 0;
        int imageCount = 0;
        int codeBlockCount = 0;
        Map<Integer, Integer> headingLevels = new TreeMap<>();
        Map<String, Integer> domainFrequency = new HashMap<>();

        @Override
        public VisitResult visitHeading(NodeContext ctx, int level, String text, String id) {
            headingCount++;
            headingLevels.put(level, headingLevels.getOrDefault(level, 0) + 1);
            return VisitResult.Continue.INSTANCE;
        }

        @Override
        public VisitResult visitLink(NodeContext ctx, String href, String text, String title) {
            linkCount++;
            if (href != null && href.startsWith("http")) {
                String domain = extractDomain(href);
                domainFrequency.put(domain, domainFrequency.getOrDefault(domain, 0) + 1);
            }
            return VisitResult.Continue.INSTANCE;
        }

        @Override
        public VisitResult visitImage(NodeContext ctx, String src, String alt, String title) {
            imageCount++;
            return VisitResult.Continue.INSTANCE;
        }

        @Override
        public VisitResult visitCodeBlock(NodeContext ctx, String lang, String code) {
            codeBlockCount++;
            return VisitResult.Continue.INSTANCE;
        }

        /**
         * Extract domain from a full URL.
         */
        private String extractDomain(String url) {
            try {
                int start = url.indexOf("://") + PROTOCOL_SEPARATOR_LENGTH;
                int end = url.indexOf("/", start);
                if (end == -1) {
                    end = url.length();
                }
                return url.substring(start, end);
            } catch (Exception e) {
                return "unknown";
            }
        }

        /**
         * Print collected statistics.
         */
        void printStats() {
            System.out.println("=== Document Analytics ===");
            System.out.println("Headings: " + headingCount);
            System.out.println("  Breakdown:");
            headingLevels.forEach((level, count) ->
                System.out.println("    H" + level + ": " + count)
            );
            System.out.println("Links: " + linkCount);
            if (!domainFrequency.isEmpty()) {
                System.out.println("  Top domains:");
                domainFrequency.forEach((domain, count) ->
                    System.out.println("    " + domain + ": " + count)
                );
            }
            System.out.println("Images: " + imageCount);
            System.out.println("Code blocks: " + codeBlockCount);
        }
    }

    public static void main(String[] args) {
        String html = """
            <article>
                <h1>Web Development Guide</h1>
                <p>Learn how to build websites with
                   <a href="https://html.spec.whatwg.org">HTML5</a> and
                   <a href="https://www.w3.org/TR/CSS/">CSS</a>.</p>

                <h2>Getting Started</h2>
                <p>Check out <a href="https://developer.mozilla.org">MDN</a>
                   for great tutorials.</p>

                <h3>Example Code</h3>
                <pre><code class="language-html">&lt;div&gt;Hello&lt;/div&gt;</code></pre>

                <h2>Resources</h2>
                <ul>
                    <li><a href="https://github.com">GitHub</a> for code hosting</li>
                    <li><a href="https://stackoverflow.com">Stack Overflow</a> for Q&amp;A</li>
                    <li><a href="https://github.com">GitHub</a> for docs</li>
                </ul>

                <h3>Images</h3>
                <img src="/logo.png" alt="Logo">
                <img src="/banner.png" alt="Banner">

                <h2>Advanced Topics</h2>
                <pre><code class="language-javascript">console.log('hello');</code></pre>
                <pre><code class="language-css">body { margin: 0; }</code></pre>
            </article>
            """;

        System.out.println("=== Converting HTML ===");
        String markdown = HtmlToMarkdown.convert(html);
        System.out.println(markdown);

        System.out.println("\n=== Analyzing HTML ===");
        DocumentAnalyticsVisitor visitor = new DocumentAnalyticsVisitor();

        visitor.visitHeading(null, 1, "Web Development Guide", null);
        visitor.visitLink(null, "https://html.spec.whatwg.org", "HTML5", null);
        visitor.visitLink(null, "https://www.w3.org/TR/CSS/", "CSS", null);
        visitor.visitHeading(null, 2, "Getting Started", null);
        visitor.visitLink(null, "https://developer.mozilla.org", "MDN", null);
        visitor.visitHeading(null, HEADING_LEVEL_3, "Example Code", null);
        visitor.visitCodeBlock(null, "html", "<div>Hello</div>");
        visitor.visitHeading(null, 2, "Resources", null);
        visitor.visitLink(null, "https://github.com", "GitHub", null);
        visitor.visitLink(null, "https://stackoverflow.com", "Stack Overflow", null);
        visitor.visitLink(null, "https://github.com", "GitHub", null);
        visitor.visitHeading(null, HEADING_LEVEL_3, "Images", null);
        visitor.visitImage(null, "/logo.png", "Logo", null);
        visitor.visitImage(null, "/banner.png", "Banner", null);
        visitor.visitHeading(null, 2, "Advanced Topics", null);
        visitor.visitCodeBlock(null, "javascript", "console.log('hello');");
        visitor.visitCodeBlock(null, "css", "body { margin: 0; }");

        visitor.printStats();
    }
}
