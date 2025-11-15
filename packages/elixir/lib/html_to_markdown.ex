defmodule HtmlToMarkdown do
  @moduledoc """
  Elixir bindings for the Rust html-to-markdown engine.
  """

  alias HtmlToMarkdown.Error
  alias HtmlToMarkdown.Native

  @type option ::
          {:wrap, boolean()}
          | {:wrap_width, pos_integer()}
          | {:heading_style, :atx | :atx_closed | :underlined}
          | {:list_indent_type, :spaces | :tabs}
          | {:newline_style, :spaces | :backslash}
          | {:code_block_style, :indented | :backticks | :tildes}
          | {:whitespace, :normalized | :strict}
          | {:convert_as_inline, boolean()}
          | {:debug, boolean()}
          | {:preprocessing,
             %{
               optional(:enabled) => boolean(),
               optional(:preset) => :minimal | :standard | :aggressive,
               optional(:remove_navigation) => boolean(),
               optional(:remove_forms) => boolean()
             }}

  @doc """
  Convert `html` to Markdown. Returns `{:ok, markdown}` or `{:error, reason}`.
  """
  @spec convert(String.t(), Enumerable.t()) :: {:ok, String.t()} | {:error, term()}
  def convert(html, opts \\ []) when is_binary(html) do
    opts = opts |> to_map() |> stringify_keys()

    case call_native(html, opts) do
      {:ok, markdown} -> {:ok, markdown}
      {:error, reason} -> {:error, reason}
    end
  end

  @doc """
  Convert `html` to Markdown, raising on error.
  """
  @spec convert!(String.t(), Enumerable.t()) :: String.t()
  def convert!(html, opts \\ []) when is_binary(html) do
    case convert(html, opts) do
      {:ok, markdown} -> markdown
      {:error, reason} -> raise Error, message: inspect(reason)
    end
  end

  defp call_native(html, %{} = opts) when map_size(opts) == 0 do
    Native.convert(html)
  end

  defp call_native(html, opts) do
    Native.convert_with_options(html, opts)
  end

  defp to_map(%{} = map), do: map
  defp to_map(list) when is_list(list), do: Map.new(list)
  defp to_map(_), do: %{}

  defp stringify_keys(value) when is_map(value) do
    value
    |> Enum.map(fn {k, v} -> {stringify_key(k), stringify_keys(v)} end)
    |> Map.new()
  end

  defp stringify_keys(value), do: value

  defp stringify_key(key) when is_atom(key), do: Atom.to_string(key)
  defp stringify_key(key) when is_binary(key), do: key
  defp stringify_key(other), do: to_string(other)
end
