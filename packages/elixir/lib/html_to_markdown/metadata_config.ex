defmodule HtmlToMarkdown.MetadataConfig do
  @moduledoc """
  Configuration for metadata extraction.

  This controls which metadata sections are extracted during `convert_with_metadata/3`.
  """

  @default_max_structured_data_size 1_000_000

  defstruct extract_document: true,
            extract_headers: true,
            extract_links: true,
            extract_images: true,
            extract_structured_data: true,
            max_structured_data_size: @default_max_structured_data_size

  @type t :: %__MODULE__{
          extract_document: boolean(),
          extract_headers: boolean(),
          extract_links: boolean(),
          extract_images: boolean(),
          extract_structured_data: boolean(),
          max_structured_data_size: pos_integer()
        }

  @spec new(t() | map() | keyword() | nil) :: t()
  def new(nil), do: %__MODULE__{}
  def new(%__MODULE__{} = cfg), do: cfg

  def new(attrs) when is_list(attrs) or is_map(attrs) do
    attrs =
      attrs
      |> Map.new()
      |> Enum.reduce(%{}, fn {key, value}, acc ->
        case normalize_key(key) do
          nil -> acc
          normalized -> Map.put(acc, normalized, value)
        end
      end)

    struct(%__MODULE__{}, attrs)
  end

  defp normalize_key(key) when is_atom(key) do
    if key in [
         :extract_document,
         :extract_headers,
         :extract_links,
         :extract_images,
         :extract_structured_data,
         :max_structured_data_size
       ],
       do: key,
       else: nil
  end

  defp normalize_key(key) when is_binary(key) do
    key
    |> String.trim()
    |> String.replace("-", "_")
    |> case do
      "extract_document" -> :extract_document
      "extract_headers" -> :extract_headers
      "extract_links" -> :extract_links
      "extract_images" -> :extract_images
      "extract_structured_data" -> :extract_structured_data
      "max_structured_data_size" -> :max_structured_data_size
      _ -> nil
    end
  end

  defp normalize_key(_), do: nil

  @doc false
  @spec to_map_result(t() | map() | keyword() | nil) :: {:ok, map()} | {:error, String.t()}
  def to_map_result(nil), do: to_map_result(%__MODULE__{})
  def to_map_result(%__MODULE__{} = cfg), do: config_to_map(cfg)

  def to_map_result(attrs) when is_list(attrs) or is_map(attrs),
    do: attrs |> new() |> config_to_map()

  defp config_to_map(%__MODULE__{} = cfg) do
    with {:ok, extract_document} <- ensure_boolean(cfg.extract_document, :extract_document),
         {:ok, extract_headers} <- ensure_boolean(cfg.extract_headers, :extract_headers),
         {:ok, extract_links} <- ensure_boolean(cfg.extract_links, :extract_links),
         {:ok, extract_images} <- ensure_boolean(cfg.extract_images, :extract_images),
         {:ok, extract_structured_data} <-
           ensure_boolean(cfg.extract_structured_data, :extract_structured_data),
         {:ok, max_size} <-
           positive_integer(cfg.max_structured_data_size, :max_structured_data_size) do
      {:ok,
       %{
         "extract_document" => extract_document,
         "extract_headers" => extract_headers,
         "extract_links" => extract_links,
         "extract_images" => extract_images,
         "extract_structured_data" => extract_structured_data,
         "max_structured_data_size" => max_size
       }}
    end
  end

  defp ensure_boolean(value, _field) when is_boolean(value), do: {:ok, value}
  defp ensure_boolean(_value, field), do: {:error, error_message(field, "must be a boolean")}

  defp positive_integer(value, _field) when is_integer(value) and value > 0, do: {:ok, value}

  defp positive_integer(value, field) when is_binary(value) do
    value
    |> String.trim()
    |> Integer.parse()
    |> case do
      {parsed, ""} when parsed > 0 -> {:ok, parsed}
      _ -> {:error, error_message(field, "must be a positive integer")}
    end
  end

  defp positive_integer(_value, field),
    do: {:error, error_message(field, "must be a positive integer")}

  defp error_message(field, message) when is_atom(field), do: "#{field}: #{message}"
  defp error_message(field, message), do: "#{field}: #{message}"
end
