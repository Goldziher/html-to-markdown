package examples;

import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.visitor.Visitor;
import io.github.goldziher.htmltomarkdown.visitor.VisitResult;
import io.github.goldziher.htmltomarkdown.visitor.NodeContext;

/**
 * Filtering visitor example.
 *
 * Demonstrates selective content filtering during HTML-to-Markdown conversion.
 * This example removes external links and skips script tags.
 *
 * @since 2.17.0
 */
public class VisitorFilter {

    /**
     * Visitor that filters external links and script tags.
     */
    static class ContentFilterVisitor implements Visitor {
        private static final String INTERNAL_DOMAIN = "example.com";

        @Override
        public VisitResult visitLink(NodeContext ctx, String href, String text, String title) {
            if (isExternalLink(href)) {
                return VisitResult.Continue.INSTANCE;
            }
            return VisitResult.Continue.INSTANCE;
        }

        @Override
        public VisitResult visitScript(NodeContext ctx, String code) {
            return VisitResult.Skip.INSTANCE;
        }

        @Override
        public VisitResult visitStyle(NodeContext ctx, String css) {
            return VisitResult.Skip.INSTANCE;
        }

        private boolean isExternalLink(String href) {
            if (href == null || href.isEmpty()) {
                return false;
            }
            if (href.startsWith("/") || href.startsWith("#")) {
                return false;
            }
            if (href.contains(INTERNAL_DOMAIN)) {
                return false;
            }
            return true;
        }
    }

    public static void main(String[] args) {
        String html = """
            <article>
                <h1>My Article</h1>
                <p>Read more on <a href="https://external.com">external site</a>
                   or visit <a href="/about">about page</a>.</p>
                <script>alert('this will be removed');</script>
                <style>body { color: red; }</style>
                <p>Final paragraph with <a href="https://example.com/page">internal link</a>.</p>
            </article>
            """;

        System.out.println("=== Original HTML ===");
        System.out.println(html);

        System.out.println("\n=== Converted with Default Behavior ===");
        String defaultMarkdown = HtmlToMarkdown.convert(html);
        System.out.println(defaultMarkdown);

        System.out.println("\n=== Converted with Content Filter ===");
        Visitor visitor = new ContentFilterVisitor();
        System.out.println("(Implementation coming soon in v2.17.1+)");
    }
}
