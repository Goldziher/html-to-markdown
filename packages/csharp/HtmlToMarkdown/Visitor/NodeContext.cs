namespace HtmlToMarkdown.Visitor;

/// <summary>
/// Context information provided for each node visited during HTML to Markdown conversion.
/// </summary>
public sealed class NodeContext
{
    /// <summary>
    /// Gets the coarse-grained node type classification.
    /// </summary>
    public NodeType NodeType { get; }

    /// <summary>
    /// Gets the raw HTML tag name (e.g., "div", "h1", "custom-element").
    /// For text nodes, this may be empty.
    /// </summary>
    public string TagName { get; }

    /// <summary>
    /// Gets the HTML attributes as an array of key-value pairs.
    /// </summary>
    public IReadOnlyList<Attribute> Attributes { get; }

    /// <summary>
    /// Gets the depth in the DOM tree (0 = root).
    /// </summary>
    public int Depth { get; }

    /// <summary>
    /// Gets the zero-based index among siblings.
    /// </summary>
    public int IndexInParent { get; }

    /// <summary>
    /// Gets the parent element's tag name, or null if this is the root element.
    /// </summary>
    public string? ParentTag { get; }

    /// <summary>
    /// Gets a value indicating whether this element is treated as inline vs block.
    /// </summary>
    public bool IsInline { get; }

    /// <summary>
    /// Initializes a new instance of the NodeContext class.
    /// </summary>
    /// <param name="nodeType">The node type classification</param>
    /// <param name="tagName">The HTML tag name</param>
    /// <param name="attributes">The element attributes</param>
    /// <param name="depth">The depth in the DOM tree</param>
    /// <param name="indexInParent">The index among siblings</param>
    /// <param name="parentTag">The parent tag name, or null</param>
    /// <param name="isInline">Whether the element is inline</param>
    public NodeContext(
        NodeType nodeType,
        string tagName,
        IReadOnlyList<Attribute> attributes,
        int depth,
        int indexInParent,
        string? parentTag,
        bool isInline)
    {
        NodeType = nodeType;
        TagName = tagName ?? throw new ArgumentNullException(nameof(tagName));
        Attributes = attributes ?? throw new ArgumentNullException(nameof(attributes));
        Depth = depth;
        IndexInParent = indexInParent;
        ParentTag = parentTag;
        IsInline = isInline;
    }

    /// <summary>
    /// Gets the first attribute value with the given key, or null if not found.
    /// </summary>
    /// <param name="key">The attribute key to find</param>
    /// <returns>The attribute value, or null if not found</returns>
    public string? GetAttribute(string key)
    {
        if (string.IsNullOrEmpty(key))
        {
            return null;
        }

        foreach (var attr in Attributes)
        {
            if (string.Equals(attr.Key, key, StringComparison.Ordinal))
            {
                return attr.Value;
            }
        }

        return null;
    }

    /// <summary>
    /// Checks if the element has an attribute with the given key.
    /// </summary>
    /// <param name="key">The attribute key to check</param>
    /// <returns>True if the attribute exists, false otherwise</returns>
    public bool HasAttribute(string key)
    {
        return GetAttribute(key) != null;
    }

    public override string ToString() => $"<{TagName} depth={Depth} parent={ParentTag}>";
}
