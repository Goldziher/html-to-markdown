package examples;

import io.github.goldziher.htmltomarkdown.HtmlToMarkdown;
import io.github.goldziher.htmltomarkdown.visitor.Visitor;
import io.github.goldziher.htmltomarkdown.visitor.VisitResult;
import io.github.goldziher.htmltomarkdown.visitor.NodeContext;

/**
 * Basic visitor pattern example.
 *
 * Demonstrates the simplest use of the visitor pattern to intercept
 * and customize HTML-to-Markdown conversion.
 *
 * @since 2.17.0
 */
public class VisitorBasic {

    /**
     * Simple visitor that uppercases all headings.
     */
    static class UppercaseHeadingVisitor implements Visitor {
        @Override
        public VisitResult visitHeading(NodeContext ctx, int level, String text, String id) {
            String prefix = "#".repeat(level);
            String output = prefix + " " + text.toUpperCase();
            return new VisitResult.Custom(output);
        }
    }

    public static void main(String[] args) {
        String html = """
            <h1>Hello World</h1>
            <p>This is a paragraph with <strong>bold</strong> text.</p>
            <h2>Section Two</h2>
            <p>More content here.</p>
            """;

        System.out.println("=== Default Conversion ===");
        String defaultMarkdown = HtmlToMarkdown.convert(html);
        System.out.println(defaultMarkdown);

        System.out.println("\n=== With Visitor (Uppercase Headings) ===");
        Visitor visitor = new UppercaseHeadingVisitor();
        System.out.println("(Implementation coming soon in v2.17.1+)");
    }
}
