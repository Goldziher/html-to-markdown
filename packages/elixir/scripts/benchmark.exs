args = System.argv()

{options, _, _} =
  OptionParser.parse(args,
    switches: [file: :string, iterations: :integer, format: :string]
  )

file = options[:file]
iterations = max(options[:iterations] || 50, 1)
format = (options[:format] || "html") |> String.downcase()

if is_nil(file) do
  IO.puts(:stderr, "Error: --file is required")
  System.halt(1)
end

if format not in ["html", "hocr"] do
  IO.puts(:stderr, "Unsupported format: #{format}")
  System.halt(1)
end

unless File.exists?(file) do
  IO.puts(:stderr, "Fixture not found: #{file}")
  System.halt(1)
end

html = File.read!(file)

_ = HtmlToMarkdown.convert(html, if(format == "hocr", do: [hocr_spatial_tables: false], else: []))

profile_output = System.get_env("HTML_TO_MARKDOWN_PROFILE_OUTPUT")
if profile_output && profile_output != "" do
  freq_env = System.get_env("HTML_TO_MARKDOWN_PROFILE_FREQUENCY") || "1000"
  frequency =
    case Integer.parse(freq_env) do
      {value, _} -> value
      :error -> 1000
    end

  HtmlToMarkdown.start_profiling(profile_output, frequency)
end

start = System.monotonic_time()
Enum.each(1..iterations, fn _ ->
  HtmlToMarkdown.convert(html, if(format == "hocr", do: [hocr_spatial_tables: false], else: []))
end)
finish = System.monotonic_time()

if profile_output && profile_output != "" do
  HtmlToMarkdown.stop_profiling()
end

elapsed_seconds = System.convert_time_unit(finish - start, :native, :microsecond) / 1_000_000
bytes_processed = byte_size(html) * iterations
ops_per_sec = iterations / elapsed_seconds
mb_per_sec = (bytes_processed / (1024 * 1024)) / elapsed_seconds

fixture = Path.basename(file)

json =
  "{\"language\":\"elixir\",\"fixture\":\"#{fixture}\"," <>
    "\"fixture_path\":\"#{file}\",\"iterations\":#{iterations}," <>
    "\"elapsed_seconds\":#{Float.round(elapsed_seconds, 8)}," <>
    "\"ops_per_sec\":#{Float.round(ops_per_sec, 4)}," <>
    "\"mb_per_sec\":#{Float.round(mb_per_sec, 4)}," <>
    "\"bytes_processed\":#{bytes_processed}}"

IO.puts(json)
