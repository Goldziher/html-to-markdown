package io.github.goldziher.htmltomarkdown.visitor;

import java.util.List;
import java.util.Objects;

/**
 * Immutable context information for a node being visited during HTML-to-Markdown conversion.
 *
 * <p>Passed to all visitor callbacks to provide metadata about the current element. All data is
 * borrowed from the Rust converter and valid only during the callback invocation.
 *
 * <p><b>Memory Safety Note:</b> Do NOT attempt to store or persist strings from this context. All
 * pointers are borrowed and become invalid after the callback returns. If you need persistent data,
 * copy the string contents immediately.
 *
 * @param nodeType the coarse-grained type classification of this node
 * @param tagName the raw HTML tag name (e.g., "div", "h1", "custom-element")
 * @param attributes unmodifiable list of attribute pairs
 * @param depth the depth in the DOM tree (0 = root)
 * @param indexInParent the zero-based index among siblings
 * @param parentTag the parent element's tag name, or null if root
 * @param isInline whether this element is treated as inline vs block
 * @since 2.17.0
 */
public record NodeContext(
    NodeType nodeType,
    String tagName,
    List<Attribute> attributes,
    int depth,
    int indexInParent,
    String parentTag,
    boolean isInline) {

  /**
   * Create a new NodeContext with validation.
   *
   * @param nodeType the node type
   * @param tagName the tag name
   * @param attributes the attributes
   * @param depth the depth
   * @param indexInParent the index in parent
   * @param parentTag the parent tag
   * @param isInline the inline flag
   * @throws NullPointerException if nodeType or tagName is null
   * @throws IllegalArgumentException if depth or indexInParent is negative
   */
  public NodeContext {
    Objects.requireNonNull(nodeType, "NodeType cannot be null");
    Objects.requireNonNull(tagName, "Tag name cannot be null");
    Objects.requireNonNull(attributes, "Attributes cannot be null");

    if (depth < 0) {
      throw new IllegalArgumentException("Depth cannot be negative");
    }
    if (indexInParent < 0) {
      throw new IllegalArgumentException("Index in parent cannot be negative");
    }

    attributes = List.copyOf(attributes);
  }

  /**
   * Check if this node is a text node.
   *
   * @return true if this is a text node
   */
  public boolean isTextNode() {
    return nodeType == NodeType.TEXT;
  }

  /**
   * Check if this node is an element node.
   *
   * @return true if this is an element node
   */
  public boolean isElement() {
    return nodeType == NodeType.ELEMENT;
  }

  /**
   * Check if this node is a heading.
   *
   * @return true if this node's type is HEADING
   */
  public boolean isHeading() {
    return nodeType == NodeType.HEADING;
  }

  /**
   * Get an attribute value by name, or null if not present.
   *
   * @param name the attribute name
   * @return the attribute value, or null if not found
   */
  public String getAttributeValue(final String name) {
    Objects.requireNonNull(name, "Attribute name cannot be null");
    return attributes.stream()
        .filter(attr -> attr.key().equals(name))
        .map(Attribute::value)
        .findFirst()
        .orElse(null);
  }

  /**
   * Check if an attribute exists.
   *
   * @param name the attribute name
   * @return true if the attribute is present
   */
  public boolean hasAttribute(final String name) {
    Objects.requireNonNull(name, "Attribute name cannot be null");
    return attributes.stream().anyMatch(attr -> attr.key().equals(name));
  }

  /**
   * Check if this node is the root element (depth == 0).
   *
   * @return true if depth is 0
   */
  public boolean isRoot() {
    return depth == 0;
  }

  /**
   * Check if this node has a parent.
   *
   * @return true if parentTag is not null
   */
  public boolean hasParent() {
    return parentTag != null;
  }

  /**
   * Return a string representation of this node context.
   *
   * @return a formatted description
   */
  @Override
  public String toString() {
    return String.format(
        "NodeContext{type=%s, tag=%s, depth=%d, index=%d, inline=%s}",
        nodeType, tagName, depth, indexInParent, isInline);
  }
}
