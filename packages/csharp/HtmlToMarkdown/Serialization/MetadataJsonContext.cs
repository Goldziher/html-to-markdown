using System.Text.Json.Serialization;
using HtmlToMarkdown.Metadata;

namespace HtmlToMarkdown.Serialization;

[JsonSourceGenerationOptions(PropertyNameCaseInsensitive = true)]
[JsonSerializable(typeof(ExtendedMetadata))]
internal partial class MetadataJsonContext : JsonSerializerContext
{
}
