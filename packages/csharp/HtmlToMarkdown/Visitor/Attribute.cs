namespace HtmlToMarkdown.Visitor;

/// <summary>
/// Represents an HTML attribute (key-value pair) in a visitor callback.
/// </summary>
public readonly struct Attribute
{
    /// <summary>
    /// Gets the attribute name (e.g., "href", "class", "id").
    /// </summary>
    public string Key { get; }

    /// <summary>
    /// Gets the attribute value.
    /// </summary>
    public string Value { get; }

    /// <summary>
    /// Initializes a new instance of the Attribute struct.
    /// </summary>
    /// <param name="key">The attribute name</param>
    /// <param name="value">The attribute value</param>
    public Attribute(string key, string value)
    {
        Key = key ?? throw new ArgumentNullException(nameof(key));
        Value = value ?? string.Empty;
    }

    public override string ToString() => $"{Key}=\"{Value}\"";
}
