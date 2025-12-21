namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Result of metadata extraction when returning raw JSON.
/// </summary>
public record MetadataExtractionJson
{
    /// <summary>
    /// Converted Markdown output.
    /// </summary>
    public string Markdown { get; init; } = string.Empty;

    /// <summary>
    /// Metadata JSON payload.
    /// </summary>
    public string MetadataJson { get; init; } = string.Empty;
}
