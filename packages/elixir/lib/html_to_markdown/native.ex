defmodule HtmlToMarkdown.Native do
  @moduledoc false

  use Rustler,
    otp_app: :html_to_markdown,
    crate: "html_to_markdown_elixir",
    path: "native/html_to_markdown_elixir",
    mode: (Mix.env() == :prod && :release) || :debug

  def convert(_html), do: :erlang.nif_error(:nif_not_loaded)
  def convert_with_options(_html, _opts), do: :erlang.nif_error(:nif_not_loaded)
end
