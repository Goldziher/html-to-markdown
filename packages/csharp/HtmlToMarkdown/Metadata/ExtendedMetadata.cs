using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Comprehensive metadata extraction result from HTML document.
/// Contains all extracted metadata types in a single structure,
/// suitable for serialization and transmission across language boundaries.
/// </summary>
public record ExtendedMetadata
{
    /// <summary>
    /// Document-level metadata (title, description, canonical, etc.).
    /// </summary>
    [JsonPropertyName("document")]
    public DocumentMetadata Document { get; init; } = new();

    /// <summary>
    /// Extracted header elements with hierarchy.
    /// </summary>
    [JsonPropertyName("headers")]
    public List<HeaderMetadata> Headers { get; init; } = [];

    /// <summary>
    /// Extracted hyperlinks with type classification.
    /// </summary>
    [JsonPropertyName("links")]
    public List<LinkMetadata> Links { get; init; } = [];

    /// <summary>
    /// Extracted images with source and dimensions.
    /// </summary>
    [JsonPropertyName("images")]
    public List<ImageMetadata> Images { get; init; } = [];

    /// <summary>
    /// Extracted structured data blocks.
    /// </summary>
    [JsonPropertyName("structured_data")]
    public List<StructuredData> StructuredData { get; init; } = [];
}
