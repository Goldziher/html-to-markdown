#!/usr/bin/env ruby
# frozen_string_literal: true

##
#   - Result conversion overhead

require 'json'
require 'benchmark'
require 'fileutils'
require 'pathname'

lib_path = File.expand_path('../packages/ruby/lib', __dir__)
$LOAD_PATH.unshift(lib_path) if File.directory?(lib_path)

begin
  require 'html-to-markdown'
rescue LoadError
  warn "Error: Could not load html-to-markdown gem"
  warn "Make sure to install with: cd packages/ruby && bundle install"
  exit 1
end

##
# Timing metrics for a test run
class TimingMetrics
  attr_reader :scenario, :html_size_bytes, :element_count, :baseline_ms,
              :visitor_ms, :overhead_ms, :overhead_percent,
              :callback_invocations, :avg_callback_time_us, :iterations

  def initialize(
    scenario:, html_size_bytes:, element_count:, baseline_ms:,
    visitor_ms:, overhead_ms:, overhead_percent:,
    callback_invocations:, avg_callback_time_us:, iterations:
  )
    @scenario = scenario
    @html_size_bytes = html_size_bytes
    @element_count = element_count
    @baseline_ms = baseline_ms
    @visitor_ms = visitor_ms
    @overhead_ms = overhead_ms
    @overhead_percent = overhead_percent
    @callback_invocations = callback_invocations
    @avg_callback_time_us = avg_callback_time_us
    @iterations = iterations
  end

  def to_h
    {
      scenario: @scenario,
      html_size_bytes: @html_size_bytes,
      element_count: @element_count,
      baseline_ms: @baseline_ms,
      visitor_ms: @visitor_ms,
      overhead_ms: @overhead_ms,
      overhead_percent: @overhead_percent,
      callback_invocations: @callback_invocations,
      avg_callback_time_us: @avg_callback_time_us,
      iterations: @iterations,
    }
  end
end

##
# Load test HTML document
def load_test_html(filename)
  filepath = File.expand_path(
    "../test_documents/html/wikipedia/#{filename}",
    __dir__
  )
  raise "Test document not found: #{filepath}" unless File.exist?(filepath)

  File.read(filepath)
end

##
# Count HTML elements by regex
def count_elements(html)
  html.scan(%r{<[^/>]+>}).length
end

##
# No-op visitor
class NoOpVisitor
  attr_reader :invocations

  def initialize
    @invocations = 0
  end

  def visit_node(node)
    @invocations += 1
  end
end

##
# Simple text extraction visitor
class SimpleVisitor
  attr_reader :invocations, :texts

  def initialize
    @invocations = 0
    @texts = []
  end

  def visit_node(node)
    @invocations += 1
    return unless node.is_a?(Hash) && node['type'] == 'text'

    text = node['content'] || ''
    @texts << text
  end
end

##
# Custom output builder visitor
class CustomOutputVisitor
  attr_reader :invocations, :output

  def initialize
    @invocations = 0
    @output = []
  end

  def visit_node(node)
    @invocations += 1
    return unless node.is_a?(Hash)

    node_type = node['type'] || ''
    tag = node['tag'] || ''

    if node_type == 'element'
      @output << "[#{tag.upcase}]"
    elsif node_type == 'text'
      content = node['content'] || ''
      @output << content
    end
  end
end

##
# Complex visitor with multiple operations
class ComplexVisitor
  attr_reader :invocations, :stats, :depths

  def initialize
    @invocations = 0
    @stats = {}
    @depths = []
  end

  def visit_node(node)
    @invocations += 1
    return unless node.is_a?(Hash)

    node_type = node['type'] || ''

    @stats[node_type] ||= 0
    @stats[node_type] += 1

    depth = node['depth'] || 0
    @depths << depth

    if node['attributes'].is_a?(Hash)
      attr_count = node['attributes'].length
      @stats['attrs_total'] ||= 0
      @stats['attrs_total'] += attr_count
    end
  end
end

##
# Benchmark conversion with visitor
def benchmark_with_visitor(html, visitor, iterations = 10)
  Benchmark.measure do
    iterations.times do
      begin
        HtmlToMarkdown.convert_with_visitor(html, visitor: visitor)
      rescue StandardError
        nil
      end
    end
  end.real * 1000
end

##
# Benchmark baseline conversion
def benchmark_baseline(html, iterations = 10)
  Benchmark.measure do
    iterations.times do
      HtmlToMarkdown.convert(html)
    end
  end.real * 1000
end

##
# Profile a specific scenario
def profile_scenario(name, html, visitor_class, iterations = 10)
  puts "\nProfiling scenario: #{name}"
  puts "  HTML size: #{html.length} bytes"
  puts "  Iterations: #{iterations}"

  element_count = count_elements(html)

  puts "  Warming up..."
  benchmark_baseline(html, 2)

  puts "  Running baseline..."
  baseline_ms = benchmark_baseline(html, iterations)
  baseline_avg = baseline_ms / iterations

  puts "  Running #{name} visitor..."
  visitor = visitor_class.new
  visitor_ms = benchmark_with_visitor(html, visitor, iterations)
  visitor_avg = visitor_ms / iterations

  overhead_ms = visitor_ms - baseline_ms
  overhead_percent = baseline_ms > 0 ? (overhead_ms / baseline_ms) * 100 : 0

  callback_count = visitor.invocations || 0
  avg_callback_time_us = callback_count > 0 ? (overhead_ms * 1000 / callback_count) : 0

  puts "  Baseline: #{format('%.2f', baseline_avg)}ms/iter"
  puts "  Visitor:  #{format('%.2f', visitor_avg)}ms/iter"
  puts "  Overhead: #{format('%.2f', overhead_ms)}ms (#{format('%.1f', overhead_percent)}%)"
  puts "  Callbacks: #{callback_count} (#{format('%.2f', avg_callback_time_us)}µs avg)"

  TimingMetrics.new(
    scenario: name,
    html_size_bytes: html.length,
    element_count: element_count,
    baseline_ms: baseline_avg,
    visitor_ms: visitor_avg,
    overhead_ms: overhead_ms,
    overhead_percent: overhead_percent,
    callback_invocations: callback_count,
    avg_callback_time_us: avg_callback_time_us,
    iterations: iterations
  )
end

##
# Parse command line arguments
def parse_args
  options = {
    scenario: 'all',
    html: 'medium',
    iterations: 10,
    output: 'visitor_profile_results',
  }

  i = 0
  while i < ARGV.length
    case ARGV[i]
    when '--scenario'
      options[:scenario] = ARGV[i + 1]
      i += 2
    when '--html'
      options[:html] = ARGV[i + 1]
      i += 2
    when '--iterations'
      options[:iterations] = ARGV[i + 1].to_i
      i += 2
    when '--output'
      options[:output] = ARGV[i + 1]
      i += 2
    else
      i += 1
    end
  end

  options
end

##
# Main entry point
def main
  options = parse_args

  FileUtils.mkdir_p(options[:output])

  html_map = {
    'small' => 'small_html.html',
    'medium' => 'medium_python.html',
    'large' => 'large_rust.html',
  }
  html_file = html_map[options[:html]] || 'medium_python.html'
  html = load_test_html(html_file)

  puts "Visitor Callback Profiling (Ruby)"
  puts "================================="
  puts "HTML: #{options[:html]} (#{html_file})"
  puts "Size: #{html.length} bytes"
  puts "Output: #{options[:output]}"

  scenarios = [
    ['no-op', NoOpVisitor],
    ['simple', SimpleVisitor],
    ['custom-output', CustomOutputVisitor],
    ['complex', ComplexVisitor],
  ]

  results = []

  scenarios.each do |scenario_name, visitor_class|
    next if options[:scenario] != 'all' && scenario_name != options[:scenario]

    metrics = profile_scenario(
      scenario_name,
      html,
      visitor_class,
      options[:iterations]
    )
    results << metrics
  end

  json_path = File.join(options[:output], 'results.json')
  output_data = {
    html_size: html.length,
    html_file: html_file,
    element_count: count_elements(html),
    timestamp: Time.now.to_i,
    results: results.map(&:to_h),
  }

  File.write(json_path, JSON.pretty_generate(output_data))
  puts "\n\nResults written to #{json_path}"

  puts "\n\nSummary"
  puts "======="
  results.each do |result|
    puts format(
      "%20s Baseline: %7.2fms, Overhead: %7.2fms (%5.1f%%), Avg callback: %7.2fµs",
      result.scenario,
      result.baseline_ms,
      result.overhead_ms,
      result.overhead_percent,
      result.avg_callback_time_us
    )
  end
end

main
