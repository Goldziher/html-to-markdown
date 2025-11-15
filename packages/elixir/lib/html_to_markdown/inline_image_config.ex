defmodule HtmlToMarkdown.InlineImageConfig do
  @moduledoc """
  Configuration for inline image extraction.
  """

  @default_limit 5 * 1_024 * 1_024

  defstruct max_decoded_size_bytes: @default_limit,
            filename_prefix: nil,
            capture_svg: true,
            infer_dimensions: false

  @type t :: %__MODULE__{
          max_decoded_size_bytes: pos_integer(),
          filename_prefix: String.t() | nil,
          capture_svg: boolean(),
          infer_dimensions: boolean()
        }

  @doc """
  Build a configuration struct from a map, keyword list, or another struct.
  """
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
    if key in [:max_decoded_size_bytes, :filename_prefix, :capture_svg, :infer_dimensions],
      do: key,
      else: nil
  end

  defp normalize_key(key) when is_binary(key) do
    key
    |> String.trim()
    |> String.replace("-", "_")
    |> case do
      "max_decoded_size_bytes" -> :max_decoded_size_bytes
      "filename_prefix" -> :filename_prefix
      "capture_svg" -> :capture_svg
      "infer_dimensions" -> :infer_dimensions
      _ -> nil
    end
  end

  defp normalize_key(_), do: nil

  @doc false
  @spec to_map(t() | map() | keyword() | nil) :: map()
  def to_map(input) do
    case to_map_result(input) do
      {:ok, cfg} -> cfg
      {:error, reason} -> raise ArgumentError, reason
    end
  end

  @doc false
  @spec to_map_result(t() | map() | keyword() | nil) :: {:ok, map()} | {:error, String.t()}
  def to_map_result(nil), do: to_map_result(%__MODULE__{})
  def to_map_result(%__MODULE__{} = cfg), do: config_to_map(cfg)

  def to_map_result(attrs) when is_list(attrs) or is_map(attrs),
    do: attrs |> new() |> config_to_map()

  defp config_to_map(%__MODULE__{} = cfg) do
    with {:ok, max_bytes} <- positive_integer(cfg.max_decoded_size_bytes, :max_decoded_size_bytes),
         {:ok, capture_svg} <- ensure_boolean(cfg.capture_svg, :capture_svg),
         {:ok, infer_dimensions} <- ensure_boolean(cfg.infer_dimensions, :infer_dimensions),
         {:ok, prefix} <- normalize_prefix(cfg.filename_prefix) do
      base = %{
        "max_decoded_size_bytes" => max_bytes,
        "capture_svg" => capture_svg,
        "infer_dimensions" => infer_dimensions
      }

      map =
        if prefix do
          Map.put(base, "filename_prefix", prefix)
        else
          base
        end

      {:ok, map}
    end
  end

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

  defp ensure_boolean(value, _field) when is_boolean(value), do: {:ok, value}
  defp ensure_boolean(_value, field), do: {:error, error_message(field, "must be a boolean")}

  defp normalize_prefix(nil), do: {:ok, nil}

  defp normalize_prefix(value) when is_binary(value) do
    trimmed = String.trim(value)

    if trimmed == "" do
      {:ok, nil}
    else
      {:ok, trimmed}
    end
  end

  defp normalize_prefix(_value), do: {:error, error_message(:filename_prefix, "must be a string")}

  defp error_message(field, message) when is_atom(field) do
    "#{field}: #{message}"
  end

  defp error_message(field, message), do: "#{field}: #{message}"
end
