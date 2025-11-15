defmodule HtmlToMarkdown.Options do
  @moduledoc """
  High-level configuration for HTML → Markdown conversion.

  Use `new/1` to build options from a map or keyword list, or construct
  the struct directly. Pass the struct to `HtmlToMarkdown.convert/2`,
  `HtmlToMarkdown.options/1`, or `HtmlToMarkdown.convert_with_inline_images/3`.
  """

  alias HtmlToMarkdown.PreprocessingOptions

  @heading_styles [:underlined, :atx, :atx_closed]
  @list_indent_types [:spaces, :tabs]
  @highlight_styles [:double_equal, :html, :bold, :none]
  @whitespace_modes [:normalized, :strict]
  @newline_styles [:spaces, :backslash]
  @code_block_styles [:indented, :backticks, :tildes]

  @option_keys [
    :heading_style,
    :list_indent_type,
    :list_indent_width,
    :bullets,
    :strong_em_symbol,
    :escape_asterisks,
    :escape_underscores,
    :escape_misc,
    :escape_ascii,
    :code_language,
    :encoding,
    :autolinks,
    :default_title,
    :keep_inline_images_in,
    :br_in_tables,
    :hocr_spatial_tables,
    :highlight_style,
    :extract_metadata,
    :whitespace_mode,
    :strip_newlines,
    :wrap,
    :wrap_width,
    :strip_tags,
    :preserve_tags,
    :convert_as_inline,
    :sub_symbol,
    :sup_symbol,
    :newline_style,
    :code_block_style,
    :preprocessing,
    :debug
  ]

  defstruct heading_style: :atx,
            list_indent_type: :spaces,
            list_indent_width: 2,
            bullets: "-*+",
            strong_em_symbol: "*",
            escape_asterisks: false,
            escape_underscores: false,
            escape_misc: false,
            escape_ascii: false,
            code_language: "",
            encoding: "utf-8",
            autolinks: true,
            default_title: false,
            keep_inline_images_in: MapSet.new(),
            br_in_tables: false,
            hocr_spatial_tables: true,
            highlight_style: :double_equal,
            extract_metadata: true,
            whitespace_mode: :normalized,
            strip_newlines: false,
            wrap: false,
            wrap_width: 80,
            strip_tags: MapSet.new(),
            preserve_tags: MapSet.new(),
            convert_as_inline: false,
            sub_symbol: "",
            sup_symbol: "",
            newline_style: :spaces,
            code_block_style: :backticks,
            preprocessing: %PreprocessingOptions{},
            debug: false

  @type heading_style :: :underlined | :atx | :atx_closed
  @type list_indent_type :: :spaces | :tabs
  @type highlight_style :: :double_equal | :html | :bold | :none
  @type whitespace_mode :: :normalized | :strict
  @type newline_style :: :spaces | :backslash
  @type code_block_style :: :indented | :backticks | :tildes

  @type t :: %__MODULE__{
          heading_style: heading_style(),
          list_indent_type: list_indent_type(),
          list_indent_width: pos_integer(),
          bullets: String.t(),
          strong_em_symbol: String.t(),
          escape_asterisks: boolean(),
          escape_underscores: boolean(),
          escape_misc: boolean(),
          escape_ascii: boolean(),
          code_language: String.t(),
          encoding: String.t(),
          autolinks: boolean(),
          default_title: boolean(),
          keep_inline_images_in: MapSet.t(),
          br_in_tables: boolean(),
          hocr_spatial_tables: boolean(),
          highlight_style: highlight_style(),
          extract_metadata: boolean(),
          whitespace_mode: whitespace_mode(),
          strip_newlines: boolean(),
          wrap: boolean(),
          wrap_width: pos_integer(),
          strip_tags: MapSet.t(),
          preserve_tags: MapSet.t(),
          convert_as_inline: boolean(),
          sub_symbol: String.t(),
          sup_symbol: String.t(),
          newline_style: newline_style(),
          code_block_style: code_block_style(),
          preprocessing: PreprocessingOptions.t(),
          debug: boolean()
        }

  @doc """
  Build options from a struct, keyword list, or map.
  """
  @spec new(t() | map() | keyword() | nil) :: t()
  def new(nil), do: %__MODULE__{}
  def new(%__MODULE__{} = opts), do: opts

  def new(attrs) when is_list(attrs) or is_map(attrs) do
    attrs =
      attrs
      |> Map.new()
      |> Enum.reduce(%{}, &normalize_entry/2)

    preprocessing = Map.get(attrs, :preprocessing, nil)
    attrs = Map.delete(attrs, :preprocessing)

    %__MODULE__{}
    |> struct(attrs)
    |> Map.update!(:preprocessing, fn existing ->
      PreprocessingOptions.new(preprocessing || existing)
    end)
  end

  defp normalize_entry({key, value}, acc) do
    case normalize_key(key) do
      nil ->
        acc

      normalized_key ->
        Map.put(acc, normalized_key, normalize_value(normalized_key, value))
    end
  end

  defp normalize_key(key) when is_atom(key), do: if(key in @option_keys, do: key, else: nil)

  defp normalize_key(key) when is_binary(key) do
    key
    |> String.trim()
    |> String.replace("-", "_")
    |> String.to_existing_atom()
  rescue
    _ -> nil
  end

  defp normalize_key(_), do: nil

  defp normalize_value(:heading_style, value), do: normalize_enum(value, @heading_styles, :atx)

  defp normalize_value(:list_indent_type, value),
    do: normalize_enum(value, @list_indent_types, :spaces)

  defp normalize_value(:highlight_style, value),
    do: normalize_enum(value, @highlight_styles, :double_equal)

  defp normalize_value(:whitespace_mode, value),
    do: normalize_enum(value, @whitespace_modes, :normalized)

  defp normalize_value(:newline_style, value), do: normalize_enum(value, @newline_styles, :spaces)

  defp normalize_value(:code_block_style, value),
    do: normalize_enum(value, @code_block_styles, :backticks)

  defp normalize_value(:preprocessing, value), do: PreprocessingOptions.new(value)

  defp normalize_value(:strong_em_symbol, value) when is_binary(value) do
    cond do
      value == "" -> "*"
      String.length(value) == 1 -> value
      true -> String.slice(value, 0, 1)
    end
  end

  defp normalize_value(:strong_em_symbol, value) when is_atom(value) do
    value |> Atom.to_string() |> normalize_value(:strong_em_symbol)
  end

  defp normalize_value(:keep_inline_images_in, value), do: normalize_set(value)
  defp normalize_value(:strip_tags, value), do: normalize_set(value)
  defp normalize_value(:preserve_tags, value), do: normalize_set(value)

  defp normalize_value(:wrap_width, value) when is_integer(value) and value > 0, do: value
  defp normalize_value(:list_indent_width, value) when is_integer(value) and value > 0, do: value
  defp normalize_value(_, value) when is_boolean(value), do: value
  defp normalize_value(_, value) when is_binary(value), do: value
  defp normalize_value(_, value) when is_integer(value), do: value
  defp normalize_value(_, value), do: value

  defp normalize_enum(value, allowed, _default) when is_atom(value) do
    if value in allowed, do: value, else: value
  end

  defp normalize_enum(value, allowed, _default) when is_binary(value) do
    normalized =
      value
      |> String.trim()
      |> String.downcase()
      |> String.replace("-", "_")

    Enum.find_value(allowed, fn atom ->
      if Atom.to_string(atom) == normalized, do: atom, else: nil
    end) || :invalid
  end

  defp normalize_enum(_value, _allowed, default), do: default

  defp normalize_set(%MapSet{} = set), do: MapSet.new(Enum.map(set, &normalize_string!/1))
  defp normalize_set(nil), do: MapSet.new()

  defp normalize_set(value) when is_list(value),
    do: MapSet.new(Enum.map(value, &normalize_string!/1))

  defp normalize_set(value) when is_binary(value), do: MapSet.new([normalize_string!(value)])
  defp normalize_set(value) when is_atom(value), do: MapSet.new([normalize_string!(value)])
  defp normalize_set(_), do: MapSet.new()

  defp normalize_string!(value) when is_binary(value), do: value
  defp normalize_string!(value) when is_atom(value), do: Atom.to_string(value)
  defp normalize_string!(value), do: to_string(value)

  @doc false
  @spec to_map(t()) :: map()
  def to_map(%__MODULE__{} = opts) do
    %{
      "heading_style" => Atom.to_string(opts.heading_style),
      "list_indent_type" => Atom.to_string(opts.list_indent_type),
      "list_indent_width" => opts.list_indent_width,
      "bullets" => opts.bullets,
      "strong_em_symbol" => opts.strong_em_symbol,
      "escape_asterisks" => opts.escape_asterisks,
      "escape_underscores" => opts.escape_underscores,
      "escape_misc" => opts.escape_misc,
      "escape_ascii" => opts.escape_ascii,
      "code_language" => opts.code_language,
      "encoding" => opts.encoding,
      "autolinks" => opts.autolinks,
      "default_title" => opts.default_title,
      "keep_inline_images_in" => mapset_to_list(opts.keep_inline_images_in),
      "br_in_tables" => opts.br_in_tables,
      "hocr_spatial_tables" => opts.hocr_spatial_tables,
      "highlight_style" => Atom.to_string(opts.highlight_style),
      "extract_metadata" => opts.extract_metadata,
      "whitespace_mode" => Atom.to_string(opts.whitespace_mode),
      "strip_newlines" => opts.strip_newlines,
      "wrap" => opts.wrap,
      "wrap_width" => opts.wrap_width,
      "strip_tags" => mapset_to_list(opts.strip_tags),
      "preserve_tags" => mapset_to_list(opts.preserve_tags),
      "convert_as_inline" => opts.convert_as_inline,
      "sub_symbol" => opts.sub_symbol,
      "sup_symbol" => opts.sup_symbol,
      "newline_style" => Atom.to_string(opts.newline_style),
      "code_block_style" => Atom.to_string(opts.code_block_style),
      "preprocessing" => PreprocessingOptions.to_map(opts.preprocessing),
      "debug" => opts.debug
    }
  end

  defp mapset_to_list(%MapSet{} = set) do
    set
    |> Enum.map(&normalize_string!/1)
    |> Enum.reject(&(&1 == ""))
  end
end
