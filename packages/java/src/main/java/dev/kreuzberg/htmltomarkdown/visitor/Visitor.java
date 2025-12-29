package dev.kreuzberg.htmltomarkdown.visitor;

import java.util.List;

/**
 * Visitor interface for customizing HTML-to-Markdown conversion.
 *
 * <p>Implement this interface to intercept and customize the conversion process. Each method is
 * called at appropriate points during tree traversal.
 *
 * <p><b>Default Behavior:</b> All methods default to returning {@link VisitResult.Continue}, which
 * instructs the converter to use its standard behavior. Override specific methods to customize
 * processing.
 *
 * <p><b>Performance Notes:</b>
 *
 * <ul>
 *   <li>{@link #visitText(NodeContext, String)} is called very frequently (100+ times per
 *       document). For performance, return {@code Continue} quickly if you're not interested in
 *       modifying the text.
 *   <li>String data in callbacks is borrowed; copy immediately if you need to persist it.
 *   <li>Callbacks must be thread-safe if the visitor is shared across threads.
 * </ul>
 *
 * @since 2.17.0
 */
public interface Visitor {

  /**
   * Called before entering any HTML element.
   *
   * @param ctx the node context
   * @return the visitor result
   */
  default VisitResult visitElementStart(final NodeContext ctx) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Called after exiting any HTML element with the default markdown output.
   *
   * @param ctx the node context
   * @param output the default markdown output
   * @return the visitor result
   */
  default VisitResult visitElementEnd(final NodeContext ctx, final String output) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit text nodes (called very frequently - 100+ times per document).
   *
   * @param ctx the node context
   * @param text the text content
   * @return the visitor result
   */
  default VisitResult visitText(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit anchor links: {@code <a href="..."></a>}.
   *
   * @param ctx the node context
   * @param href the link destination
   * @param text the link text
   * @param title the link title (may be null)
   * @return the visitor result
   */
  default VisitResult visitLink(
      final NodeContext ctx, final String href, final String text, final String title) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit images: {@code <img src="..." />}.
   *
   * @param ctx the node context
   * @param src the image source URL
   * @param alt the alt text (may be empty)
   * @param title the image title (may be null)
   * @return the visitor result
   */
  default VisitResult visitImage(
      final NodeContext ctx, final String src, final String alt, final String title) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit heading elements ({@code <h1>} through {@code <h6>}).
   *
   * @param ctx the node context
   * @param level the heading level (1-6)
   * @param text the heading text
   * @param id the heading ID attribute (may be null)
   * @return the visitor result
   */
  default VisitResult visitHeading(
      final NodeContext ctx, final int level, final String text, final String id) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit code blocks: {@code <pre>} and {@code <code>} elements.
   *
   * @param ctx the node context
   * @param lang the language identifier (may be null)
   * @param code the code content
   * @return the visitor result
   */
  default VisitResult visitCodeBlock(final NodeContext ctx, final String lang, final String code) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit inline code elements ({@code <code>}).
   *
   * @param ctx the node context
   * @param code the code content
   * @return the visitor result
   */
  default VisitResult visitCodeInline(final NodeContext ctx, final String code) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit list items.
   *
   * @param ctx the node context
   * @param ordered true if parent list is ordered
   * @param marker the bullet/number marker
   * @param text the item text
   * @return the visitor result
   */
  default VisitResult visitListItem(
      final NodeContext ctx, final boolean ordered, final String marker, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Called before processing a list ({@code <ul>} or {@code <ol>}).
   *
   * @param ctx the node context
   * @param ordered true if this is an ordered list
   * @return the visitor result
   */
  default VisitResult visitListStart(final NodeContext ctx, final boolean ordered) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Called after processing a list (closing {@code </ul>} or {@code </ol>}).
   *
   * @param ctx the node context
   * @param ordered true if this is an ordered list
   * @param output the default markdown output
   * @return the visitor result
   */
  default VisitResult visitListEnd(
      final NodeContext ctx, final boolean ordered, final String output) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Called before processing a table ({@code <table>}).
   *
   * @param ctx the node context
   * @return the visitor result
   */
  default VisitResult visitTableStart(final NodeContext ctx) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit table rows ({@code <tr>}) elements.
   *
   * @param ctx the node context
   * @param cells the cell contents as a list
   * @param isHeader true if this is a header row
   * @return the visitor result
   */
  default VisitResult visitTableRow(
      final NodeContext ctx, final List<String> cells, final boolean isHeader) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Called after processing a table element (closing {@code </table>}).
   *
   * @param ctx the node context
   * @param output the default markdown output
   * @return the visitor result
   */
  default VisitResult visitTableEnd(final NodeContext ctx, final String output) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit blockquote ({@code <blockquote>}) elements.
   *
   * @param ctx the node context
   * @param content the blockquote content
   * @param depth the blockquote nesting depth
   * @return the visitor result
   */
  default VisitResult visitBlockquote(
      final NodeContext ctx, final String content, final int depth) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit strong/bold elements ({@code <strong>}, {@code <b>}).
   *
   * @param ctx the node context
   * @param text the element text
   * @return the visitor result
   */
  default VisitResult visitStrong(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit emphasis/italic elements ({@code <em>}, {@code <i>}).
   *
   * @param ctx the node context
   * @param text the element text
   * @return the visitor result
   */
  default VisitResult visitEmphasis(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit strikethrough elements ({@code <s>}, {@code <del>}, {@code <strike>}).
   *
   * @param ctx the node context
   * @param text the element text
   * @return the visitor result
   */
  default VisitResult visitStrikethrough(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit underline elements ({@code <u>}, {@code <ins>}).
   *
   * @param ctx the node context
   * @param text the element text
   * @return the visitor result
   */
  default VisitResult visitUnderline(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit subscript elements ({@code <sub>}).
   *
   * @param ctx the node context
   * @param text the element text
   * @return the visitor result
   */
  default VisitResult visitSubscript(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit superscript elements ({@code <sup>}).
   *
   * @param ctx the node context
   * @param text the element text
   * @return the visitor result
   */
  default VisitResult visitSuperscript(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit mark/highlight ({@code <mark>}) elements.
   *
   * @param ctx the node context
   * @param text the element text
   * @return the visitor result
   */
  default VisitResult visitMark(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit line break ({@code <br>}) elements.
   *
   * @param ctx the node context
   * @return the visitor result
   */
  default VisitResult visitLineBreak(final NodeContext ctx) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit horizontal rule ({@code <hr>}) elements.
   *
   * @param ctx the node context
   * @return the visitor result
   */
  default VisitResult visitHorizontalRule(final NodeContext ctx) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit custom elements (web components) or unknown tags.
   *
   * @param ctx the node context
   * @param tagName the tag name
   * @param html the raw HTML
   * @return the visitor result
   */
  default VisitResult visitCustomElement(
      final NodeContext ctx, final String tagName, final String html) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit definition list ({@code <dl>}) element.
   *
   * @param ctx the node context
   * @return the visitor result
   */
  default VisitResult visitDefinitionListStart(final NodeContext ctx) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit definition term ({@code <dt>}) element.
   *
   * @param ctx the node context
   * @param text the term text
   * @return the visitor result
   */
  default VisitResult visitDefinitionTerm(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit definition description ({@code <dd>}) element.
   *
   * @param ctx the node context
   * @param text the description text
   * @return the visitor result
   */
  default VisitResult visitDefinitionDescription(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Called after processing a definition list (closing {@code </dl>}).
   *
   * @param ctx the node context
   * @param output the default markdown output
   * @return the visitor result
   */
  default VisitResult visitDefinitionListEnd(final NodeContext ctx, final String output) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit form ({@code <form>}) elements.
   *
   * @param ctx the node context
   * @param action the form action (may be null)
   * @param method the form method (may be null)
   * @return the visitor result
   */
  default VisitResult visitForm(final NodeContext ctx, final String action, final String method) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit input ({@code <input>}) elements.
   *
   * @param ctx the node context
   * @param inputType the input type
   * @param name the input name (may be null)
   * @param value the input value (may be null)
   * @return the visitor result
   */
  default VisitResult visitInput(
      final NodeContext ctx, final String inputType, final String name, final String value) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit button ({@code <button>}) elements.
   *
   * @param ctx the node context
   * @param text the button text
   * @return the visitor result
   */
  default VisitResult visitButton(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit audio ({@code <audio>}) elements.
   *
   * @param ctx the node context
   * @param src the audio source URL (may be null)
   * @return the visitor result
   */
  default VisitResult visitAudio(final NodeContext ctx, final String src) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit video ({@code <video>}) elements.
   *
   * @param ctx the node context
   * @param src the video source URL (may be null)
   * @return the visitor result
   */
  default VisitResult visitVideo(final NodeContext ctx, final String src) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit iframe ({@code <iframe>}) elements.
   *
   * @param ctx the node context
   * @param src the iframe source URL (may be null)
   * @return the visitor result
   */
  default VisitResult visitIframe(final NodeContext ctx, final String src) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit details ({@code <details>}) elements.
   *
   * @param ctx the node context
   * @param open true if the details element is open
   * @return the visitor result
   */
  default VisitResult visitDetails(final NodeContext ctx, final boolean open) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit summary ({@code <summary>}) elements.
   *
   * @param ctx the node context
   * @param text the summary text
   * @return the visitor result
   */
  default VisitResult visitSummary(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit figure ({@code <figure>}) elements.
   *
   * @param ctx the node context
   * @return the visitor result
   */
  default VisitResult visitFigureStart(final NodeContext ctx) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Visit figcaption ({@code <figcaption>}) elements.
   *
   * @param ctx the node context
   * @param text the caption text
   * @return the visitor result
   */
  default VisitResult visitFigcaption(final NodeContext ctx, final String text) {
    return VisitResult.Continue.INSTANCE;
  }

  /**
   * Called after processing a figure element (closing {@code </figure>}).
   *
   * @param ctx the node context
   * @param output the default markdown output
   * @return the visitor result
   */
  default VisitResult visitFigureEnd(final NodeContext ctx, final String output) {
    return VisitResult.Continue.INSTANCE;
  }
}
