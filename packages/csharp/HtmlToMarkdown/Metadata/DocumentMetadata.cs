using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Document-level metadata extracted from &lt;head&gt; and top-level elements.
/// Contains all metadata typically used by search engines, social media platforms,
/// and browsers for document indexing and presentation.
/// </summary>
public record DocumentMetadata
{
    /// <summary>
    /// Document title from &lt;title&gt; tag.
    /// </summary>
    [JsonPropertyName("title")]
    public string? Title { get; init; }

    /// <summary>
    /// Document description from &lt;meta name="description"&gt; tag.
    /// </summary>
    [JsonPropertyName("description")]
    public string? Description { get; init; }

    /// <summary>
    /// Document keywords from &lt;meta name="keywords"&gt; tag, split on commas.
    /// </summary>
    [JsonPropertyName("keywords")]
    public List<string> Keywords { get; init; } = [];

    /// <summary>
    /// Document author from &lt;meta name="author"&gt; tag.
    /// </summary>
    [JsonPropertyName("author")]
    public string? Author { get; init; }

    /// <summary>
    /// Canonical URL from &lt;link rel="canonical"&gt; tag.
    /// </summary>
    [JsonPropertyName("canonical_url")]
    public string? CanonicalUrl { get; init; }

    /// <summary>
    /// Base URL from &lt;base href=""&gt; tag for resolving relative URLs.
    /// </summary>
    [JsonPropertyName("base_href")]
    public string? BaseHref { get; init; }

    /// <summary>
    /// Document language from &lt;html lang&gt; attribute.
    /// </summary>
    [JsonPropertyName("language")]
    public string? Language { get; init; }

    /// <summary>
    /// Document text direction from &lt;html dir&gt; attribute.
    /// </summary>
    [JsonPropertyName("text_direction")]
    public TextDirection? TextDirection { get; init; }

    /// <summary>
    /// Open Graph metadata (og:* properties) for social media.
    /// Keys like "title", "description", "image", "url", etc.
    /// </summary>
    [JsonPropertyName("open_graph")]
    public Dictionary<string, string> OpenGraph { get; init; } = [];

    /// <summary>
    /// Twitter Card metadata (twitter:* properties).
    /// Keys like "card", "site", "creator", "title", "description", "image", etc.
    /// </summary>
    [JsonPropertyName("twitter_card")]
    public Dictionary<string, string> TwitterCard { get; init; } = [];

    /// <summary>
    /// Additional meta tags not covered by specific fields.
    /// Keys are meta name/property attributes, values are content.
    /// </summary>
    [JsonPropertyName("meta_tags")]
    public Dictionary<string, string> MetaTags { get; init; } = [];
}
