#!/usr/bin/env ruby
# frozen_string_literal: true

require 'optparse'
require 'time'

$LOAD_PATH.unshift(File.expand_path('../lib', __dir__))
require 'html_to_markdown'

def json_escape(value)
  value.to_s.gsub(/["\\\n\r]/) do |char|
    case char
    when '"', '\\'
      "\\#{char}"
    when "\n"
      '\\n'
    when "\r"
      '\\r'
    end
  end
end

options = {
  iterations: 50,
  format: 'html',
  scenario: 'convert-default'
}

OptionParser.new do |parser|
  parser.banner = 'ruby benchmark.rb --file path/to/fixture.html [--iterations 200]'

  parser.on('--file FILE', 'HTML fixture to convert repeatedly') do |file|
    options[:file] = file
  end

  parser.on('--iterations N', Integer, 'Number of conversion iterations (default: 50)') do |n|
    options[:iterations] = n.positive? ? n : 1
  end

  parser.on('--scenario SCENARIO', 'Scenario to benchmark') do |scenario|
    options[:scenario] = scenario
  end

  parser.on('--format FORMAT', 'Fixture format (html or hocr)') do |format|
    options[:format] = format.downcase
  end
end.parse!

fixture = options.fetch(:file) do
  warn 'Missing --file parameter'
  exit 1
end

unless File.exist?(fixture)
  warn "Fixture not found: #{fixture}"
  exit 1
end

unless %w[html hocr].include?(options[:format])
  warn "Unsupported format: #{options[:format]}"
  exit 1
end

supported_scenarios = %w[
  convert-default
  convert-options
  inline-images-default
  inline-images-options
  metadata-default
  metadata-options
]
unless supported_scenarios.include?(options[:scenario])
  warn "Unsupported scenario: #{options[:scenario]}"
  exit 1
end

html = File.binread(fixture)
html.force_encoding(Encoding::UTF_8)
html.freeze
iterations = options[:iterations]
conversion_options = options[:format] == 'hocr' ? { hocr_spatial_tables: false } : {}

SCENARIO_RUNNERS = {
  'convert-default' => ->(html, _options) { HtmlToMarkdown.convert(html) },
  'convert-options' => ->(html, options) { HtmlToMarkdown.convert(html, options) },
  'inline-images-default' => ->(html, _options) { HtmlToMarkdown.convert_with_inline_images(html, nil, nil) },
  'inline-images-options' => ->(html, options) { HtmlToMarkdown.convert_with_inline_images(html, options, nil) },
  'metadata-default' => ->(html, _options) { HtmlToMarkdown.convert_with_metadata(html, nil, nil) },
  'metadata-options' => ->(html, options) { HtmlToMarkdown.convert_with_metadata(html, options, nil) }
}.freeze

def run_scenario(html, scenario, options)
  runner = SCENARIO_RUNNERS.fetch(scenario) { raise ArgumentError, "Unsupported scenario: #{scenario}" }
  runner.call(html, options)
end

run_scenario(html, options[:scenario], conversion_options)

profile_output = ENV.fetch('HTML_TO_MARKDOWN_PROFILE_OUTPUT', nil)
if profile_output && HtmlToMarkdown.respond_to?(:start_profiling)
  freq = Integer(ENV.fetch('HTML_TO_MARKDOWN_PROFILE_FREQUENCY', '1000'), 10)
  HtmlToMarkdown.start_profiling(profile_output, freq)
end

start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
iterations.times { run_scenario(html, options[:scenario], conversion_options) }
elapsed = Process.clock_gettime(Process::CLOCK_MONOTONIC) - start

HtmlToMarkdown.stop_profiling if profile_output && HtmlToMarkdown.respond_to?(:stop_profiling)

payload_size_bytes = html.bytesize
bytes_processed = payload_size_bytes * iterations
ops_per_sec = iterations / elapsed
mb_per_sec = (bytes_processed.to_f / (1024 * 1024)) / elapsed

payload = %({
  "language":"ruby",
  "fixture":"#{json_escape(File.basename(fixture))}",
  "fixture_path":"#{json_escape(fixture)}",
  "scenario":"#{json_escape(options[:scenario])}",
  "iterations":#{iterations},
  "elapsed_seconds":#{format('%.8f', elapsed)},
  "ops_per_sec":#{format('%.4f', ops_per_sec)},
  "mb_per_sec":#{format('%.4f', mb_per_sec)},
  "bytes_processed":#{bytes_processed},
  "payload_size_bytes":#{payload_size_bytes}
})

puts payload.strip
