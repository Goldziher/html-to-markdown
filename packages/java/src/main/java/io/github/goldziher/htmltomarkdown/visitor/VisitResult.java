package io.github.goldziher.htmltomarkdown.visitor;

import java.util.Objects;

/**
 * Sealed interface representing the result of a visitor callback.
 *
 * <p>Communicates to the converter how to proceed after visiting a node. Use the factory methods to
 * create appropriate result types.
 *
 * <p><b>Example usage:</b>
 *
 * <pre>{@code
 * // Continue with default conversion
 * return VisitResult.Continue.INSTANCE;
 *
 * // Replace with custom markdown
 * return VisitResult.Custom.create("**custom**");
 *
 * // Skip this element and children
 * return VisitResult.Skip.INSTANCE;
 *
 * // Preserve original HTML
 * return VisitResult.PreserveHtml.INSTANCE;
 *
 * // Report an error
 * return VisitResult.Error.create("Invalid element");
 * }</pre>
 *
 * @since 2.17.0
 */
public sealed interface VisitResult {

  /**
   * Continue with default conversion behavior for this node.
   *
   * <p>The converter will apply its standard conversion logic and ignore any custom output or error
   * fields.
   *
   * @since 2.17.0
   */
  record Continue() implements VisitResult {
    /** Singleton instance. */
    public static final Continue INSTANCE = new Continue();

    @Override
    public String toString() {
      return "VisitResult.Continue";
    }
  }

  /**
   * Replace the element's output with custom Markdown.
   *
   * <p>The provided markdown string will be used instead of the converter's default output. The
   * caller is responsible for allocating memory; the converter will free it after use.
   *
   * @param customOutput the custom Markdown content
   * @since 2.17.0
   */
  record Custom(String customOutput) implements VisitResult {
    /**
     * Create a new Custom result with validation.
     *
     * @param customOutput the custom markdown output
     * @throws NullPointerException if customOutput is null
     */
    public Custom {
      Objects.requireNonNull(customOutput, "Custom output cannot be null");
    }

    @Override
    public String toString() {
      return "VisitResult.Custom[length=" + customOutput.length() + "]";
    }
  }

  /**
   * Skip this element and all its children entirely.
   *
   * <p>The element and its subtree will be omitted from the output.
   *
   * @since 2.17.0
   */
  record Skip() implements VisitResult {
    /** Singleton instance. */
    public static final Skip INSTANCE = new Skip();

    @Override
    public String toString() {
      return "VisitResult.Skip";
    }
  }

  /**
   * Preserve the original HTML instead of converting.
   *
   * <p>The raw HTML of this element will be included in the output as-is.
   *
   * @since 2.17.0
   */
  record PreserveHtml() implements VisitResult {
    /** Singleton instance. */
    public static final PreserveHtml INSTANCE = new PreserveHtml();

    @Override
    public String toString() {
      return "VisitResult.PreserveHtml";
    }
  }

  /**
   * Stop conversion and report an error.
   *
   * <p>The conversion process will halt immediately, and the error message will be returned to the
   * caller as a ConversionException.
   *
   * @param errorMessage the error description
   * @since 2.17.0
   */
  record Error(String errorMessage) implements VisitResult {
    /**
     * Create a new Error result with validation.
     *
     * @param errorMessage the error message
     * @throws NullPointerException if errorMessage is null
     */
    public Error {
      Objects.requireNonNull(errorMessage, "Error message cannot be null");
    }

    @Override
    public String toString() {
      return "VisitResult.Error[" + errorMessage + "]";
    }
  }
}
