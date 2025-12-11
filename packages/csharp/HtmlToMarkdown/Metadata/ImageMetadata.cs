using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Metadata;

/// <summary>
/// Custom JSON converter for image dimensions.
/// Handles deserialization of dimensions from array format.
/// </summary>
public class DimensionsJsonConverter : JsonConverter<(uint Width, uint Height)?>
{
    public override (uint Width, uint Height)? Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        if (reader.TokenType == JsonTokenType.Null)
        {
            return null;
        }

        if (reader.TokenType == JsonTokenType.StartArray)
        {
            reader.Read(); // Read first element
            if (!reader.TryGetUInt32(out uint width))
            {
                throw new JsonException("Expected width as uint in dimensions array");
            }

            reader.Read(); // Read second element
            if (!reader.TryGetUInt32(out uint height))
            {
                throw new JsonException("Expected height as uint in dimensions array");
            }

            reader.Read(); // Read end of array
            return (width, height);
        }

        throw new JsonException("Dimensions must be null or an array");
    }

    public override void Write(Utf8JsonWriter writer, (uint Width, uint Height)? value, JsonSerializerOptions options)
    {
        if (value == null)
        {
            writer.WriteNullValue();
        }
        else
        {
            writer.WriteStartArray();
            writer.WriteNumberValue(value.Value.Width);
            writer.WriteNumberValue(value.Value.Height);
            writer.WriteEndArray();
        }
    }
}

/// <summary>
/// Image metadata with source and dimensions.
/// Captures &lt;img&gt; elements and inline &lt;svg&gt; elements with metadata
/// for image analysis and optimization.
/// </summary>
public class ImageMetadata
{
    /// <summary>
    /// Image source (URL, data URI, or SVG content identifier).
    /// </summary>
    [JsonPropertyName("src")]
    public string Src { get; set; } = string.Empty;

    /// <summary>
    /// Alternative text from alt attribute (for accessibility).
    /// </summary>
    [JsonPropertyName("alt")]
    public string? Alt { get; set; }

    /// <summary>
    /// Title attribute (often shown as tooltip).
    /// </summary>
    [JsonPropertyName("title")]
    public string? Title { get; set; }

    /// <summary>
    /// Image dimensions as (width, height) if available.
    /// </summary>
    [JsonPropertyName("dimensions")]
    [JsonConverter(typeof(DimensionsJsonConverter))]
    public (uint Width, uint Height)? Dimensions { get; set; }

    /// <summary>
    /// Image type classification.
    /// </summary>
    [JsonPropertyName("image_type")]
    public ImageType ImageType { get; set; }

    /// <summary>
    /// Additional HTML attributes.
    /// </summary>
    [JsonPropertyName("attributes")]
    public Dictionary<string, string> Attributes { get; set; } = [];
}
