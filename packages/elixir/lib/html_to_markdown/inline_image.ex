defmodule HtmlToMarkdown.InlineImage do
  @moduledoc """
  Represents an inline image extracted during conversion.
  """

  @enforce_keys [:data, :format, :source, :attributes]
  defstruct data: <<>>,
            format: "png",
            filename: nil,
            description: nil,
            dimensions: nil,
            source: "img_data_uri",
            attributes: %{}

  @type t :: %__MODULE__{
          data: binary(),
          format: String.t(),
          filename: String.t() | nil,
          description: String.t() | nil,
          dimensions: {non_neg_integer(), non_neg_integer()} | nil,
          source: String.t(),
          attributes: map()
        }
end
