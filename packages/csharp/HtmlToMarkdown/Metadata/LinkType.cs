using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Link classification based on href value and document context.
/// Used to categorize links during extraction for filtering and analysis.
/// </summary>
[JsonConverter(typeof(JsonStringEnumConverter))]
public enum LinkType
{
    /// <summary>
    /// Anchor link within same document (href starts with #).
    /// </summary>
    [JsonPropertyName("anchor")]
    Anchor,

    /// <summary>
    /// Internal link within same domain.
    /// </summary>
    [JsonPropertyName("internal")]
    Internal,

    /// <summary>
    /// External link to different domain.
    /// </summary>
    [JsonPropertyName("external")]
    External,

    /// <summary>
    /// Email link (mailto:).
    /// </summary>
    [JsonPropertyName("email")]
    Email,

    /// <summary>
    /// Phone link (tel:).
    /// </summary>
    [JsonPropertyName("phone")]
    Phone,

    /// <summary>
    /// Other protocol or unclassifiable link type.
    /// </summary>
    [JsonPropertyName("other")]
    Other,
}
