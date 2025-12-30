defmodule HtmlToMarkdownTestApp.MixProject do
  use Mix.Project

  def project do
    [
      app: :html_to_markdown_test_app,
      version: "1.0.0",
      elixir: "~> 1.15",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [extra_applications: [:logger]]
  end

  defp deps do
    [
      {:html_to_markdown, "~> 2.19.2"}
    ]
  end
end
