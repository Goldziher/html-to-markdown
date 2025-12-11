using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Image source classification for proper handling and processing.
/// Determines whether an image is embedded (data URI), inline SVG, external, or relative.
/// </summary>
[JsonConverter(typeof(JsonStringEnumConverter))]
public enum ImageType
{
    /// <summary>
    /// Data URI embedded image (base64 or other encoding).
    /// </summary>
    [JsonPropertyName("data_uri")]
    DataUri,

    /// <summary>
    /// Inline SVG element.
    /// </summary>
    [JsonPropertyName("inline_svg")]
    InlineSvg,

    /// <summary>
    /// External image URL (http/https).
    /// </summary>
    [JsonPropertyName("external")]
    External,

    /// <summary>
    /// Relative image path.
    /// </summary>
    [JsonPropertyName("relative")]
    Relative,
}
