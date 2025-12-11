using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Result of metadata extraction from HTML conversion.
/// Contains both the converted markdown and extracted metadata.
/// </summary>
public record MetadataExtraction
{
    /// <summary>
    /// The converted Markdown string.
    /// </summary>
    [JsonPropertyName("markdown")]
    public string Markdown { get; init; } = string.Empty;

    /// <summary>
    /// Extracted metadata from the HTML document.
    /// </summary>
    [JsonPropertyName("metadata")]
    public ExtendedMetadata Metadata { get; init; } = new();
}
