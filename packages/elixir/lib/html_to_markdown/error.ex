defmodule HtmlToMarkdown.Error do
  @moduledoc """
  Raised by `HtmlToMarkdown.convert!/2` when the native converter returns an error.
  """

  defexception [:message]
end
