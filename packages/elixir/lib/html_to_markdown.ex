defmodule HtmlToMarkdown do
  @moduledoc """
  High-level Elixir interface for the Rust html-to-markdown engine.
  """

  alias HtmlToMarkdown.{
    Error,
    InlineImage,
    InlineImageConfig,
    InlineImageWarning,
    MetadataConfig,
    Native,
    Options
  }

  @type options_input :: Options.t() | map() | keyword() | nil
  @type inline_config_input :: InlineImageConfig.t() | map() | keyword() | nil
  @type metadata_config_input :: MetadataConfig.t() | map() | keyword() | nil

  @doc """
  Convert HTML to Markdown.

  The `options` argument accepts an `%HtmlToMarkdown.Options{}` struct,
  a map/keyword list with option keys, or `nil` (defaults).
  """
  @spec convert(String.t(), options_input()) :: {:ok, String.t()} | {:error, term()}
  def convert(html, options \\ nil) when is_binary(html) do
    options_map = normalize_options(options)

    case call_convert(html, options_map) do
      {:ok, markdown} -> {:ok, markdown}
      {:error, reason} -> {:error, reason}
    end
  end

  @doc """
  Convert HTML to Markdown and raise on failure.
  """
  @spec convert!(String.t(), options_input()) :: String.t()
  def convert!(html, options \\ nil) do
    case convert(html, options) do
      {:ok, markdown} -> markdown
      {:error, reason} -> raise Error, message: inspect(reason)
    end
  end

  @doc """
  Convert HTML using a reusable options handle.
  """
  @spec convert_with_options(String.t(), reference()) :: {:ok, String.t()} | {:error, term()}
  def convert_with_options(html, handle) when is_binary(html) do
    Native.convert_with_handle(html, handle)
  end

  @doc """
  Variant of `convert_with_options/2` that raises on failure.
  """
  @spec convert_with_options!(String.t(), reference()) :: String.t()
  def convert_with_options!(html, handle) do
    case convert_with_options(html, handle) do
      {:ok, markdown} -> markdown
      {:error, reason} -> raise Error, message: inspect(reason)
    end
  end

  @doc """
  Convert HTML and collect inline image assets.

  Returns `{:ok, markdown, inline_images, warnings}`.
  """
  @spec convert_with_inline_images(String.t(), options_input(), inline_config_input()) ::
          {:ok, String.t(), [InlineImage.t()], [InlineImageWarning.t()]}
          | {:error, term()}
  def convert_with_inline_images(html, options \\ nil, inline_config \\ nil)
      when is_binary(html) do
    options_map = normalize_options(options) || %{}

    with {:ok, config_map} <- InlineImageConfig.to_map_result(inline_config),
         {:ok, {markdown, images, warnings}} <-
           Native.convert_with_inline_images(html, options_map, config_map) do
      {:ok, markdown, Enum.map(images, &into_inline_image/1), Enum.map(warnings, &into_warning/1)}
    else
      {:error, reason} -> {:error, reason}
    end
  end

  @doc """
  Bang variant of `convert_with_inline_images/3`.
  """
  @spec convert_with_inline_images!(String.t(), options_input(), inline_config_input()) ::
          {String.t(), [InlineImage.t()], [InlineImageWarning.t()]}
  def convert_with_inline_images!(html, options \\ nil, inline_config \\ nil) do
    case convert_with_inline_images(html, options, inline_config) do
      {:ok, markdown, images, warnings} -> {markdown, images, warnings}
      {:error, reason} -> raise Error, message: inspect(reason)
    end
  end

  @doc """
  Convert HTML to Markdown and extract metadata.

  Returns `{:ok, markdown, metadata}`.
  """
  @spec convert_with_metadata(String.t(), options_input(), metadata_config_input()) ::
          {:ok, String.t(), map()}
          | {:error, term()}
  def convert_with_metadata(html, options \\ nil, metadata_config \\ nil) when is_binary(html) do
    options_map = normalize_options(options) || %{}

    with {:ok, metadata_map} <- MetadataConfig.to_map_result(metadata_config),
         {:ok, {markdown, metadata}} <-
           Native.convert_with_metadata(html, options_map, metadata_map) do
      {:ok, markdown, normalize_metadata(metadata)}
    else
      {:error, reason} -> {:error, reason}
    end
  end

  @doc """
  Bang variant of `convert_with_metadata/3`.
  """
  @spec convert_with_metadata!(String.t(), options_input(), metadata_config_input()) ::
          {String.t(), map()}
  def convert_with_metadata!(html, options \\ nil, metadata_config \\ nil) do
    case convert_with_metadata(html, options, metadata_config) do
      {:ok, markdown, metadata} -> {markdown, metadata}
      {:error, reason} -> raise Error, message: inspect(reason)
    end
  end

  @doc """
  Create a reusable options handle (opaque reference).

  The handle can be passed to `convert_with_options/2`.
  """
  @spec options(options_input()) :: reference()
  def options(opts \\ nil) do
    opts_map = normalize_options(opts) || %{}

    case Native.create_options_handle(opts_map) do
      {:ok, handle} ->
        handle

      {:error, reason} ->
        raise Error, message: inspect(reason)
    end
  end

  @doc """
  Start Rust-side profiling and write a flamegraph to the given output path.
  """
  @spec start_profiling(String.t(), integer()) :: :ok | {:error, term()}
  def start_profiling(output_path, frequency \\ 1000) when is_binary(output_path) do
    Native.start_profiling(output_path, frequency)
  end

  @doc """
  Stop Rust-side profiling and flush the flamegraph.
  """
  @spec stop_profiling() :: :ok | {:error, term()}
  def stop_profiling do
    Native.stop_profiling()
  end

  defp call_convert(html, nil), do: Native.convert(html)

  defp call_convert(html, options) when options == %{}, do: Native.convert(html)

  defp call_convert(html, options), do: Native.convert_with_options_map(html, options)

  defp normalize_options(nil), do: nil

  defp normalize_options(options) do
    options
    |> Options.new()
    |> Options.to_map()
  end

  defp into_inline_image(map) do
    data = fetch(map, :data)
    format = fetch(map, :format)
    filename = fetch(map, :filename)
    description = fetch(map, :description)
    dimensions = normalize_dimensions(fetch(map, :dimensions))
    source = normalize_source(fetch(map, :source))
    attributes = fetch(map, :attributes) || %{}

    %InlineImage{
      data: data,
      format: format,
      filename: filename,
      description: description,
      dimensions: dimensions,
      source: source,
      attributes: attributes
    }
  end

  defp into_warning(map) do
    index = fetch(map, :index)
    message = fetch(map, :message)
    %InlineImageWarning{index: index, message: message}
  end

  defp fetch(map, key) when is_atom(key) do
    Map.get(map, key) || Map.get(map, Atom.to_string(key))
  end

  defp normalize_source(value) when is_atom(value), do: Atom.to_string(value)
  defp normalize_source(value) when is_binary(value), do: value
  defp normalize_source(_), do: "img_data_uri"

  defp normalize_dimensions({width, height}) when is_integer(width) and is_integer(height),
    do: {width, height}

  defp normalize_dimensions([width, height]) when is_integer(width) and is_integer(height),
    do: {width, height}

  defp normalize_dimensions(_), do: nil

  defp normalize_metadata(value) when is_list(value), do: Enum.map(value, &normalize_metadata/1)

  defp normalize_metadata(value) when is_map(value) do
    value
    |> Enum.map(fn {k, v} ->
      key =
        cond do
          is_atom(k) -> Atom.to_string(k)
          is_binary(k) -> k
          true -> to_string(k)
        end

      {key, normalize_metadata(v)}
    end)
    |> Map.new()
  end

  defp normalize_metadata(value), do: value
end
