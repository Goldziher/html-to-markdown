defmodule HtmlToMarkdown.InlineImageWarning do
  @moduledoc """
  Warning emitted during inline image extraction.
  """

  @enforce_keys [:index, :message]
  defstruct index: 0, message: ""

  @type t :: %__MODULE__{index: non_neg_integer(), message: String.t()}
end
