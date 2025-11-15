Mix.Task.run("app.start")

alias HtmlToMarkdown

{opts, _} =
  OptionParser.parse!(System.argv(),
    strict: [file: :string, iterations: :integer, format: :string],
    aliases: [f: :file]
  )

file = opts[:file] || raise ArgumentError, "--file is required"
iterations = max(opts[:iterations] || 50, 1)
format = opts[:format] || "html"

html = File.read!(file)
options =
  case String.downcase(format) do
    "hocr" -> %{hocr_spatial_tables: false}
    _ -> %{}
  end

handle = HtmlToMarkdown.options(options)
HtmlToMarkdown.convert_with_options!(html, handle)

start = System.monotonic_time(:nanosecond)
for _ <- 1..iterations do
  HtmlToMarkdown.convert_with_options!(html, handle)
end
elapsed_seconds = (System.monotonic_time(:nanosecond) - start) / 1_000_000_000.0

bytes_processed = byte_size(html) * iterations
ops_per_sec = iterations / elapsed_seconds
mb_per_sec = (bytes_processed / 1_048_576.0) / elapsed_seconds

result = %{
  language: "elixir",
  fixture: Path.basename(file),
  fixture_path: Path.expand(file),
  iterations: iterations,
  elapsed_seconds: elapsed_seconds,
  ops_per_sec: ops_per_sec,
  mb_per_sec: mb_per_sec,
  bytes_processed: bytes_processed
}

IO.puts(Jason.encode!(result))
