using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Structured data block (JSON-LD, Microdata, or RDFa).
/// Represents machine-readable structured data found in the document.
/// JSON-LD blocks are collected as raw JSON strings for flexibility.
/// </summary>
public record StructuredData
{
    /// <summary>
    /// Type of structured data (JSON-LD, Microdata, RDFa).
    /// </summary>
    [JsonPropertyName("data_type")]
    public StructuredDataType DataType { get; init; }

    /// <summary>
    /// Raw JSON string (for JSON-LD) or serialized representation.
    /// </summary>
    [JsonPropertyName("raw_json")]
    public string RawJson { get; init; } = string.Empty;

    /// <summary>
    /// Schema type if detectable (e.g., "Article", "Event", "Product").
    /// </summary>
    [JsonPropertyName("schema_type")]
    public string? SchemaType { get; init; }
}
