defmodule HtmlToMarkdownTestApp.ComprehensiveTest do
  use ExUnit.Case

  defp load_fixtures(filename) do
    path = Path.join([__DIR__, "..", "..", "fixtures", filename])
    {:ok, content} = File.read(path)
    Jason.decode!(content)
  end

  describe "basic HTML conversions" do
    setup do
      {:ok, fixtures: load_fixtures("basic-html.json")}
    end

    test "all basic fixtures", %{fixtures: fixtures} do
      Enum.each(fixtures, fn fixture ->
        {:ok, result} = HtmlToMarkdown.convert(fixture["html"], fixture["options"] || %{})
        expected = String.trim(fixture["expectedMarkdown"])
        actual = String.trim(result)

        assert actual == expected, """
        Test case: #{fixture["name"]}
        Expected: #{expected}
        Got: #{actual}
        """
      end)
    end
  end
end
