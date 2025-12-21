defmodule HtmlToMarkdown.Native do
  @moduledoc false

  @features System.get_env("HTML_TO_MARKDOWN_CARGO_FEATURES", "")
            |> String.split(",", trim: true)

  use Rustler,
    otp_app: :html_to_markdown,
    crate: "html_to_markdown_elixir",
    path: "native/html_to_markdown_elixir",
    mode: (Mix.env() == :prod && :release) || :debug,
    features: @features

  def convert(_html), do: :erlang.nif_error(:nif_not_loaded)
  def convert_with_options_map(_html, _options), do: :erlang.nif_error(:nif_not_loaded)
  def convert_with_handle(_html, _handle), do: :erlang.nif_error(:nif_not_loaded)
  def create_options_handle(_options), do: :erlang.nif_error(:nif_not_loaded)
  def convert_with_inline_images(_html, _options, _config), do: :erlang.nif_error(:nif_not_loaded)
  def convert_with_metadata(_html, _options, _config), do: :erlang.nif_error(:nif_not_loaded)
  def start_profiling(_output, _frequency), do: :erlang.nif_error(:nif_not_loaded)
  def stop_profiling, do: :erlang.nif_error(:nif_not_loaded)
end
