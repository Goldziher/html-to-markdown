package dev.kreuzberg.htmltomarkdown.metadata;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;
import java.util.Objects;

/**
 * Structured data block (JSON-LD, Microdata, or RDFa).
 *
 * <p>Represents machine-readable structured data found in the document. JSON-LD blocks are
 * collected as raw JSON strings for flexibility.
 *
 * @param dataType Type of structured data (JSON-LD, Microdata, RDFa)
 * @param rawJson Raw JSON string (for JSON-LD) or serialized representation
 * @param schemaType Schema type if detectable (e.g., "Article", "Event", "Product")
 * @since 2.13.0
 */
@JsonInclude(JsonInclude.Include.NON_NULL)
public record StructuredData(
    @JsonProperty("data_type") String dataType,
    @JsonProperty("raw_json") String rawJson,
    @JsonProperty("schema_type") String schemaType) {

  /**
   * Construct a StructuredData record.
   *
   * @param dataType the structured data type
   * @param rawJson the raw JSON string
   * @param schemaType the optional schema type
   */
  public StructuredData {
    Objects.requireNonNull(dataType, "dataType cannot be null");
    Objects.requireNonNull(rawJson, "rawJson cannot be null");
  }

  /**
   * Check if this is JSON-LD structured data.
   *
   * @return true if data type is json_ld
   */
  public boolean isJsonLd() {
    return StructuredDataType.JSON_LD.getValue().equals(dataType);
  }

  /**
   * Check if this is Microdata structured data.
   *
   * @return true if data type is microdata
   */
  public boolean isMicrodata() {
    return StructuredDataType.MICRODATA.getValue().equals(dataType);
  }

  /**
   * Check if this is RDFa structured data.
   *
   * @return true if data type is rdfa
   */
  public boolean isRdfa() {
    return StructuredDataType.RDFA.getValue().equals(dataType);
  }

  /**
   * Create StructuredData for JSON-LD.
   *
   * @param jsonContent the raw JSON content
   * @return a new StructuredData with JSON-LD type
   */
  public static StructuredData jsonLd(String jsonContent) {
    return new StructuredData(StructuredDataType.JSON_LD.getValue(), jsonContent, null);
  }

  /**
   * Create StructuredData for Microdata.
   *
   * @param content the microdata content
   * @return a new StructuredData with microdata type
   */
  public static StructuredData microdata(String content) {
    return new StructuredData(StructuredDataType.MICRODATA.getValue(), content, null);
  }

  /**
   * Create StructuredData for RDFa.
   *
   * @param content the RDFa content
   * @return a new StructuredData with RDFa type
   */
  public static StructuredData rdfa(String content) {
    return new StructuredData(StructuredDataType.RDFA.getValue(), content, null);
  }
}
