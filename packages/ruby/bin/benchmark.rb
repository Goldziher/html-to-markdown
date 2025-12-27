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
  scenario: 'convert-default',
  visitor: nil
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

  parser.on('--visitor VISITOR', 'Visitor type (noop, simple, custom, complex)') do |visitor|
    options[:visitor] = visitor if %w[noop simple custom complex].include?(visitor)
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

# Visitor factory functions
def create_noop_visitor
  {
    visit_text: proc { |_ctx, _text| 'continue' },
    visit_heading: proc { |_ctx, _level, _text, _id| 'continue' },
    visit_paragraph: proc { |_ctx, _text| 'continue' },
    visit_link: proc { |_ctx, _href, _text, _title| 'continue' },
    visit_image: proc { |_ctx, _src, _alt, _title| 'continue' },
    visit_strong: proc { |_ctx, _text| 'continue' },
    visit_em: proc { |_ctx, _text| 'continue' },
    visit_code: proc { |_ctx, _text| 'continue' },
    visit_br: proc { |_ctx| 'continue' }
  }
end

def create_simple_visitor
  {
    text_count: 0,
    link_count: 0,
    image_count: 0,
    visit_text: proc { |_ctx, _text| 'continue' },
    visit_heading: proc { |_ctx, _level, _text, _id| 'continue' },
    visit_paragraph: proc { |_ctx, _text| 'continue' },
    visit_link: proc { |_ctx, _href, _text, _title| 'continue' },
    visit_image: proc { |_ctx, _src, _alt, _title| 'continue' },
    visit_strong: proc { |_ctx, _text| 'continue' },
    visit_em: proc { |_ctx, _text| 'continue' },
    visit_code: proc { |_ctx, _text| 'continue' },
    visit_br: proc { |_ctx| 'continue' }
  }
end

def create_custom_visitor
  {
    visit_text: proc { |_ctx, _text| 'continue' },
    visit_heading: proc { |_ctx, _level, _text, _id| 'continue' },
    visit_paragraph: proc { |_ctx, _text| 'continue' },
    visit_link: proc { |_ctx, href, text, _title| ['custom', "LINK[#{text}](#{href})"] },
    visit_image: proc { |_ctx, src, alt, _title| ['custom', "![#{alt}](#{src})"] },
    visit_strong: proc { |_ctx, _text| 'continue' },
    visit_em: proc { |_ctx, _text| 'continue' },
    visit_code: proc { |_ctx, _text| 'continue' },
    visit_br: proc { |_ctx| 'continue' }
  }
end

def create_complex_visitor
  {
    texts: 0,
    links: 0,
    images: 0,
    headings: 0,
    visit_text: proc { |_ctx, _text| 'continue' },
    visit_heading: proc { |_ctx, _level, _text, _id| 'continue' },
    visit_paragraph: proc { |_ctx, _text| 'continue' },
    visit_link: proc { |_ctx, href, text, _title| ['custom', "[#{text}](#{href})"] },
    visit_image: proc { |_ctx, _src, _alt, _title| 'skip' },
    visit_strong: proc { |_ctx, _text| 'continue' },
    visit_em: proc { |_ctx, _text| 'continue' },
    visit_code: proc { |_ctx, _text| 'continue' },
    visit_br: proc { |_ctx| 'continue' }
  }
end

html = File.binread(fixture)
html.force_encoding(Encoding::UTF_8)
html.freeze
iterations = options[:iterations]
conversion_options = options[:format] == 'hocr' ? { hocr_spatial_tables: false } : {}
options_handle = if %w[convert-options inline-images-options metadata-options].include?(options[:scenario])
                   HtmlToMarkdown.options(conversion_options)
                 end

# Create visitor if specified
visitor = nil
if options[:visitor]
  visitor_creators = {
    'noop' => method(:create_noop_visitor),
    'simple' => method(:create_simple_visitor),
    'custom' => method(:create_custom_visitor),
    'complex' => method(:create_complex_visitor)
  }
  creator = visitor_creators[options[:visitor]]
  visitor = creator.call if creator
end

SCENARIO_RUNNERS = {
  'convert-default' => ->(html, _options, _handle, _visitor) { HtmlToMarkdown.convert(html) },
  'convert-options' => lambda do |html, _options, handle, _visitor|
    raise ArgumentError, 'options handle required' unless handle

    HtmlToMarkdown.convert_with_options(html, handle)
  end,
  'inline-images-default' => lambda { |html, _options, _handle, _visitor|
    HtmlToMarkdown.convert_with_inline_images(html, nil, nil)
  },
  'inline-images-options' => lambda do |html, _options, handle, _visitor|
    raise ArgumentError, 'options handle required' unless handle

    HtmlToMarkdown.convert_with_inline_images_handle(html, handle, nil)
  end,
  'metadata-default' => ->(html, _options, _handle, _visitor) { HtmlToMarkdown.convert_with_metadata(html, nil, nil) },
  'metadata-options' => lambda do |html, _options, handle, _visitor|
    raise ArgumentError, 'options handle required' unless handle

    HtmlToMarkdown.convert_with_metadata_handle(html, handle, nil)
  end
}.freeze

def run_scenario(html, scenario, options, handle, visitor = nil)
  if visitor
    HtmlToMarkdown.convert_with_visitor(html, nil, visitor)
  else
    runner = SCENARIO_RUNNERS.fetch(scenario) { raise ArgumentError, "Unsupported scenario: #{scenario}" }
    runner.call(html, options, handle, visitor)
  end
end

run_scenario(html, options[:scenario], conversion_options, options_handle, visitor)

profile_output = ENV.fetch('HTML_TO_MARKDOWN_PROFILE_OUTPUT', nil)
if profile_output && HtmlToMarkdown.respond_to?(:start_profiling)
  freq = Integer(ENV.fetch('HTML_TO_MARKDOWN_PROFILE_FREQUENCY', '1000'), 10)
  HtmlToMarkdown.start_profiling(profile_output, freq)
end

start = Process.clock_gettime(Process::CLOCK_MONOTONIC)
iterations.times { run_scenario(html, options[:scenario], conversion_options, options_handle, visitor) }
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
