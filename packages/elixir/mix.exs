defmodule HtmlToMarkdown.MixProject do
  use Mix.Project

  @version "2.14.6"
  @source_url "https://github.com/Goldziher/html-to-markdown"

  def project do
    [
      app: :html_to_markdown,
      version: @version,
      elixir: "~> 1.19",
      start_permanent: Mix.env() == :prod,
      elixirc_paths: elixirc_paths(Mix.env()),
      deps: deps(),
      description: "High-performance HTML to Markdown converter with a Rust core",
      package: package(),
      docs: docs(),
      source_url: @source_url
    ]
  end

  def application do
    [extra_applications: [:logger]]
  end

  defp deps do
    [
      {:jason, "~> 1.4", runtime: false},
      {:rustler, "~> 0.33.0", runtime: false},
      {:credo, "~> 1.7", only: [:dev, :test], runtime: false},
      {:ex_doc, "~> 0.34", only: :dev, runtime: false}
    ]
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{GitHub: @source_url},
      files: ~w(lib native mix.exs README.md .formatter.exs)
    ]
  end

  defp docs do
    [
      main: "HtmlToMarkdown",
      source_url: @source_url,
      extras: ["README.md"]
    ]
  end

  defp elixirc_paths(:test), do: ["lib", "test/support"]
  defp elixirc_paths(_), do: ["lib"]
end
