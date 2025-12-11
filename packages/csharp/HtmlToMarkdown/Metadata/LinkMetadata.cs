using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Hyperlink metadata with categorization and attributes.
/// Represents &lt;a&gt; elements with parsed href values, text content, and link type classification.
/// </summary>
public record LinkMetadata
{
    /// <summary>
    /// The href URL value.
    /// </summary>
    [JsonPropertyName("href")]
    public string Href { get; init; } = string.Empty;

    /// <summary>
    /// Link text content (normalized, concatenated if mixed with elements).
    /// </summary>
    [JsonPropertyName("text")]
    public string Text { get; init; } = string.Empty;

    /// <summary>
    /// Optional title attribute (often shown as tooltip).
    /// </summary>
    [JsonPropertyName("title")]
    public string? Title { get; init; }

    /// <summary>
    /// Link type classification.
    /// </summary>
    [JsonPropertyName("link_type")]
    public LinkType LinkType { get; init; }

    /// <summary>
    /// Rel attribute values (e.g., "nofollow", "stylesheet", "canonical").
    /// </summary>
    [JsonPropertyName("rel")]
    public List<string> Rel { get; init; } = [];

    /// <summary>
    /// Additional HTML attributes.
    /// </summary>
    [JsonPropertyName("attributes")]
    public Dictionary<string, string> Attributes { get; init; } = [];

    /// <summary>
    /// Classify a link based on href value.
    /// </summary>
    /// <param name="href">The href attribute value.</param>
    /// <returns>Appropriate LinkType based on protocol and content.</returns>
    public static LinkType ClassifyLink(string href)
    {
        if (href.StartsWith('#'))
        {
            return LinkType.Anchor;
        }

        if (href.StartsWith("mailto:"))
        {
            return LinkType.Email;
        }

        if (href.StartsWith("tel:"))
        {
            return LinkType.Phone;
        }

        if (href.StartsWith("http://") || href.StartsWith("https://"))
        {
            return LinkType.External;
        }

        if (href.StartsWith('/') || href.StartsWith("../") || href.StartsWith("./"))
        {
            return LinkType.Internal;
        }

        return LinkType.Other;
    }
}
