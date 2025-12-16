using System.Reflection;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace HtmlToMarkdown.Serialization;

/// <summary>
/// Enum JSON converter which uses <see cref="JsonPropertyNameAttribute"/> values on enum members.
/// This allows deserializing snake_case values (e.g. <c>json_ld</c>) emitted by the Rust FFI layer.
/// </summary>
/// <typeparam name="TEnum">The enum type</typeparam>
public sealed class JsonPropertyNameEnumConverter<TEnum> : JsonConverter<TEnum>
    where TEnum : struct, Enum
{
    private static readonly IReadOnlyDictionary<string, TEnum> FromString = BuildFromString();
    private static readonly IReadOnlyDictionary<TEnum, string> ToStringMap = BuildToString();

    public override TEnum Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        if (reader.TokenType != JsonTokenType.String)
        {
            throw new JsonException($"Expected string for {typeof(TEnum).Name}, got {reader.TokenType}.");
        }

        var value = reader.GetString();
        if (string.IsNullOrEmpty(value))
        {
            throw new JsonException($"Empty value for {typeof(TEnum).Name}.");
        }

        if (FromString.TryGetValue(value, out var enumValue))
        {
            return enumValue;
        }

        if (FromString.TryGetValue(value.ToLowerInvariant(), out enumValue))
        {
            return enumValue;
        }

        throw new JsonException($"Unknown {typeof(TEnum).Name} value: {value}");
    }

    public override void Write(Utf8JsonWriter writer, TEnum value, JsonSerializerOptions options)
    {
        if (ToStringMap.TryGetValue(value, out var str))
        {
            writer.WriteStringValue(str);
            return;
        }

        writer.WriteStringValue(value.ToString());
    }

    private static IReadOnlyDictionary<string, TEnum> BuildFromString()
    {
        var map = new Dictionary<string, TEnum>(StringComparer.Ordinal);

        static void AddKey(IDictionary<string, TEnum> target, string? key, TEnum value)
        {
            if (string.IsNullOrWhiteSpace(key))
            {
                return;
            }

            target[key] = value;
            target[key.ToLowerInvariant()] = value;
        }

        var enumType = typeof(TEnum);
        foreach (var name in Enum.GetNames(enumType))
        {
            var field = enumType.GetField(name, BindingFlags.Public | BindingFlags.Static);
            if (field == null)
            {
                continue;
            }

            var enumValue = (TEnum)field.GetValue(null)!;
            var attr = field.GetCustomAttribute<JsonPropertyNameAttribute>();
            AddKey(map, attr?.Name, enumValue);
            AddKey(map, name, enumValue);
        }
        return map;
    }

    private static IReadOnlyDictionary<TEnum, string> BuildToString()
    {
        var map = new Dictionary<TEnum, string>();
        var enumType = typeof(TEnum);
        foreach (var name in Enum.GetNames(enumType))
        {
            var field = enumType.GetField(name, BindingFlags.Public | BindingFlags.Static);
            if (field == null)
            {
                continue;
            }

            var enumValue = (TEnum)field.GetValue(null)!;
            var attr = field.GetCustomAttribute<JsonPropertyNameAttribute>();
            map[enumValue] = attr?.Name ?? name;
        }
        return map;
    }
}
