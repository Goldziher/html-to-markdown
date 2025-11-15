defmodule HtmlToMarkdown.PreprocessingOptions do
  @moduledoc """
  Configuration for HTML preprocessing before conversion.
  """

  @type preset :: :minimal | :standard | :aggressive

  @preset_strings %{
    minimal: "minimal",
    standard: "standard",
    aggressive: "aggressive"
  }

  @preset_atoms Map.new(@preset_strings, fn {k, v} -> {v, k} end)

  defstruct enabled: true,
            preset: :standard,
            remove_navigation: true,
            remove_forms: true

  @type t :: %__MODULE__{
          enabled: boolean(),
          preset: preset(),
          remove_navigation: boolean(),
          remove_forms: boolean()
        }

  @doc """
  Build preprocessing options from a map/keyword list/struct.
  """
  @spec new(t() | map() | keyword() | nil) :: t()
  def new(nil), do: %__MODULE__{}
  def new(%__MODULE__{} = opts), do: opts

  def new(attrs) when is_list(attrs) or is_map(attrs) do
    attrs = attrs |> Map.new() |> Enum.reduce(%{}, &normalize_entry/2)
    struct(%__MODULE__{}, attrs)
  end

  defp normalize_entry({key, value}, acc) do
    case normalize_key(key) do
      nil ->
        acc

      normalized_key ->
        Map.put(acc, normalized_key, normalize_value(normalized_key, value))
    end
  end

  defp normalize_key(key) when is_atom(key) do
    if key in [:enabled, :preset, :remove_navigation, :remove_forms], do: key, else: nil
  end

  defp normalize_key(key) when is_binary(key) do
    key
    |> String.trim()
    |> String.replace("-", "_")
    |> String.to_existing_atom()
  rescue
    _ -> nil
  end

  defp normalize_key(_), do: nil

  defp normalize_value(:preset, value) when value in [:minimal, :standard, :aggressive], do: value

  defp normalize_value(:preset, value) when is_binary(value) do
    normalized =
      value
      |> String.trim()
      |> String.downcase()
      |> String.replace("-", "_")

    Map.get(@preset_atoms, normalized, :invalid)
  end

  defp normalize_value(:preset, value), do: value

  defp normalize_value(_, value) when is_boolean(value), do: value
  defp normalize_value(_, value) when value in [0, 1], do: value == 1
  defp normalize_value(_, _value), do: true

  @doc false
  @spec to_map(t()) :: map()
  def to_map(%__MODULE__{} = opts) do
    preset =
      cond do
        is_atom(opts.preset) ->
          Map.get(@preset_strings, opts.preset, Atom.to_string(opts.preset))

        is_binary(opts.preset) ->
          opts.preset

        true ->
          to_string(opts.preset)
      end

    %{
      "enabled" => opts.enabled,
      "preset" => preset,
      "remove_navigation" => opts.remove_navigation,
      "remove_forms" => opts.remove_forms
    }
  end
end
