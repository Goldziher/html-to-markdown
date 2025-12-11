using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Structured data format type.
/// Identifies the schema/format used for structured data markup.
/// </summary>
[JsonConverter(typeof(JsonStringEnumConverter))]
public enum StructuredDataType
{
    /// <summary>
    /// JSON-LD (JSON for Linking Data) script blocks.
    /// </summary>
    [JsonPropertyName("json_ld")]
    JsonLd,

    /// <summary>
    /// HTML5 Microdata attributes (itemscope, itemtype, itemprop).
    /// </summary>
    [JsonPropertyName("microdata")]
    Microdata,

    /// <summary>
    /// RDF in Attributes (RDFa) markup.
    /// </summary>
    [JsonPropertyName("rdfa")]
    RDFa,
}
