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
            // Create custom output with uppercase text
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

        // Convert with default behavior
        System.out.println("=== Default Conversion ===");
        String defaultMarkdown = HtmlToMarkdown.convert(html);
        System.out.println(defaultMarkdown);

        // Convert with visitor - uppercase headings
        System.out.println("\n=== With Visitor (Uppercase Headings) ===");
        Visitor visitor = new UppercaseHeadingVisitor();
        // Note: convertWithVisitor would be implemented in HtmlToMarkdown
        // String customMarkdown = HtmlToMarkdown.convertWithVisitor(html, visitor);
        // System.out.println(customMarkdown);
        System.out.println("(Implementation coming soon in v2.17.1+)");
    }
}
