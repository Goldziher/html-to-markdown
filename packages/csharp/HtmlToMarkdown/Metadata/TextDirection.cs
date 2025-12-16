using System.Text.Json.Serialization;
using HtmlToMarkdown.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Represents text directionality of document content.
/// Corresponds to the HTML `dir` attribute and `bdi` element directionality.
/// </summary>
[JsonConverter(typeof(JsonPropertyNameEnumConverter<TextDirection>))]
public enum TextDirection
{
    /// <summary>
    /// Left-to-right text flow (default for Latin scripts).
    /// </summary>
    [JsonPropertyName("ltr")]
    LeftToRight,

    /// <summary>
    /// Right-to-left text flow (Hebrew, Arabic, Urdu, etc.).
    /// </summary>
    [JsonPropertyName("rtl")]
    RightToLeft,

    /// <summary>
    /// Automatic directionality detection.
    /// </summary>
    [JsonPropertyName("auto")]
    Auto,
}
