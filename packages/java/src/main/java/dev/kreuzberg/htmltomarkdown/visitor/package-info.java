/**
 * Visitor pattern support for HTML-to-Markdown conversion with Panama FFI.
 *
 * <p>This package provides a high-level Java interface for implementing custom HTML-to-Markdown
 * conversion logic via the visitor pattern. It enables fine-grained control over which elements are
 * converted and how they appear in the output.
 *
 * <p><b>Core Types:</b>
 *
 * <ul>
 *   <li>{@link dev.kreuzberg.htmltomarkdown.visitor.Visitor} - Main visitor interface
 *   <li>{@link dev.kreuzberg.htmltomarkdown.visitor.NodeContext} - Context information for visited
 *       nodes
 *   <li>{@link dev.kreuzberg.htmltomarkdown.visitor.VisitResult} - Result types controlling
 *       conversion behavior
 *   <li>{@link dev.kreuzberg.htmltomarkdown.visitor.NodeType} - Enumeration of node types
 *   <li>{@link dev.kreuzberg.htmltomarkdown.visitor.Attribute} - HTML attribute pairs
 * </ul>
 *
 * <p><b>Example - Filtering Elements:</b>
 *
 * <pre>{@code
 * class MyVisitor implements Visitor {
 *   @Override
 *   public VisitResult visitLink(NodeContext ctx, String href,
 *       String text, String title) {
 *     // Skip links to external domains
 *     if (!href.startsWith("/")
 *         && !href.contains("example.com")) {
 *       return VisitResult.Skip.INSTANCE;
 *     }
 *     return VisitResult.Continue.INSTANCE;
 *   }
 * }
 *
 * String html = "<p>Visit <a href=\"https://evil.com\">here</a>"
 *     + " or <a href=\"/about\">our site</a></p>";
 * String markdown = HtmlToMarkdown.convertWithVisitor(html,
 *     new MyVisitor());
 * // Output: Visit or our site
 * }</pre>
 *
 * <p><b>Example - Custom Formatting:</b>
 *
 * <pre>{@code
 * class HighlightVisitor implements Visitor {
 *   @Override
 *   public VisitResult visitHeading(NodeContext ctx, int level,
 *       String text, String id) {
 *     // Wrap headings with custom syntax
 *     return VisitResult.Custom.create(">>> " + text + " <<<");
 *   }
 * }
 * }</pre>
 *
 * <p><b>Performance Considerations:</b>
 *
 * <ul>
 *   <li>{@link dev.kreuzberg.htmltomarkdown.visitor.Visitor#visitText visitText()} is called very
 *       frequently. Return {@link
 *       dev.kreuzberg.htmltomarkdown.visitor.VisitResult.Continue.INSTANCE} quickly unless you need
 *       to modify text.
 *   <li>String data in callbacks is borrowed from Rust and valid only during the callback. Copy
 *       strings immediately if you need to store them.
 *   <li>Callbacks run synchronously during traversal. Avoid blocking operations.
 * </ul>
 *
 * <p><b>Thread Safety:</b> If you share a visitor instance across threads, ensure all callback
 * methods are thread-safe. The underlying Rust converter is thread-safe, but Java callback
 * implementations must manage their own synchronization.
 *
 * @since 2.17.0
 */
package dev.kreuzberg.htmltomarkdown.visitor;
