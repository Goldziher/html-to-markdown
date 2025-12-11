using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Header element metadata with hierarchy tracking.
/// Captures heading elements (h1-h6) with their text content, identifiers,
/// and position in the document structure.
/// </summary>
public record HeaderMetadata
{
    /// <summary>
    /// Header level: 1 (h1) through 6 (h6).
    /// </summary>
    [JsonPropertyName("level")]
    public byte Level { get; init; }

    /// <summary>
    /// Normalized text content of the header.
    /// </summary>
    [JsonPropertyName("text")]
    public string Text { get; init; } = string.Empty;

    /// <summary>
    /// HTML id attribute if present.
    /// </summary>
    [JsonPropertyName("id")]
    public string? Id { get; init; }

    /// <summary>
    /// Document tree depth at the header element.
    /// </summary>
    [JsonPropertyName("depth")]
    public ulong Depth { get; init; }

    /// <summary>
    /// Byte offset in original HTML document.
    /// </summary>
    [JsonPropertyName("html_offset")]
    public ulong HtmlOffset { get; init; }

    /// <summary>
    /// Validates that the header level is within valid range (1-6).
    /// </summary>
    /// <returns>true if level is 1-6, false otherwise.</returns>
    public bool IsValid() => Level >= 1 && Level <= 6;
}
