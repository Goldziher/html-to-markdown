#!/usr/bin/env ruby
# frozen_string_literal: true

# SEO Metadata Extraction Example (Ruby)
#
# Demonstrates how to extract document metadata including title, description,
# author, canonical URL, Open Graph tags, and Twitter cards for SEO analysis
# and social media optimization.

require 'html_to_markdown'

def extract_seo_metadata(html)
  markdown, metadata = HtmlToMarkdown.convert_with_metadata(html)
  doc = metadata[:document]

  {
    title: doc[:title],
    description: doc[:description],
    keywords: doc[:keywords] || [],
    author: doc[:author],
    language: doc[:language],
    canonical_url: doc[:canonical_url],
    text_direction: doc[:text_direction],
    open_graph: doc[:open_graph] || {},
    twitter_card: doc[:twitter_card] || {},
    markdown: markdown,
    header_count: metadata[:headers]&.length || 0,
    link_count: metadata[:links]&.length || 0,
    image_count: metadata[:images]&.length || 0,
  }
end

def main
  html = <<~HTML
    <html lang="en">
      <head>
        <title>10 Rust Performance Optimization Tips</title>
        <meta name="description" content="Learn practical techniques to optimize Rust code for production.">
        <meta name="keywords" content="Rust, performance, optimization, systems programming">
        <meta name="author" content="Alice Johnson">
        <link rel="canonical" href="https://example.com/rust-performance-tips">
        <meta property="og:title" content="10 Rust Performance Optimization Tips">
        <meta property="og:description" content="Expert tips for making your Rust code faster.">
        <meta property="og:image" content="https://example.com/images/rust-performance.jpg">
        <meta property="og:url" content="https://example.com/rust-performance-tips">
        <meta property="og:type" content="article">
        <meta name="twitter:card" content="summary_large_image">
        <meta name="twitter:creator" content="@alicedeveloper">
        <meta name="twitter:title" content="10 Rust Performance Optimization Tips">
        <meta name="twitter:image" content="https://example.com/images/rust-performance.jpg">
      </head>
      <body>
        <h1>10 Rust Performance Optimization Tips</h1>
        <p>Written by Alice Johnson • Published 2025-01-15</p>

        <h2>Introduction</h2>
        <p>Rust is already fast, but there are techniques to make it even faster. In this guide, we'll explore 10 practical tips for optimizing Rust code in production environments.</p>

        <h2>1. Use Release Mode for Benchmarks</h2>
        <p>Always compile with <code>--release</code> when measuring performance. Debug builds are much slower due to lack of optimizations.</p>

        <h2>2. Profile Your Code</h2>
        <p>Use tools like <code>cargo-flamegraph</code> and <code>perf</code> to identify bottlenecks. Don't guess where time is spent.</p>

        <h2>3. Reduce Allocations</h2>
        <p>Heap allocations are expensive. Use stack-allocated types (<code>Vec::with_capacity</code>, <code>String::with_capacity</code>) when you know the size upfront.</p>

        <h2>External Resources</h2>
        <p>Learn more at <a href="https://docs.rust-embedded.org/book/">The Embedded Rust Book</a> and <a href="https://doc.rust-lang.org/book/">The Rust Book</a>.</p>

        <h2>Author Links</h2>
        <p>Find me on <a href="https://twitter.com/alicedeveloper">Twitter</a>, <a href="https://github.com/alicedeveloper">GitHub</a>, or <a href="mailto:alice@example.com">email me</a>.</p>

        <img src="https://example.com/images/rust-logo.png" alt="Rust programming language logo" width="200" height="200">
      </body>
    </html>
  HTML

  seo = extract_seo_metadata(html)

  puts "=" * 80
  puts "SEO METADATA EXTRACTION EXAMPLE"
  puts "=" * 80
  puts

  # Document metadata
  puts "DOCUMENT METADATA"
  puts "-" * 80
  puts "Title:           #{seo[:title]}"
  puts "Description:     #{seo[:description]}"
  puts "Keywords:        #{seo[:keywords].empty? ? 'None' : seo[:keywords].join(', ')}"
  puts "Author:          #{seo[:author]}"
  puts "Language:        #{seo[:language]}"
  puts "Canonical URL:   #{seo[:canonical_url]}"
  puts "Text Direction:  #{seo[:text_direction] || 'None'}"
  puts

  # Open Graph metadata
  puts "OPEN GRAPH METADATA (Social Media)"
  puts "-" * 80
  if seo[:open_graph].any?
    seo[:open_graph].each do |key, value|
      puts "#{key.ljust(20)} #{value}"
    end
  else
    puts "No Open Graph metadata found"
  end
  puts

  # Twitter Card metadata
  puts "TWITTER CARD METADATA"
  puts "-" * 80
  if seo[:twitter_card].any?
    seo[:twitter_card].each do |key, value|
      puts "#{key.ljust(20)} #{value}"
    end
  else
    puts "No Twitter Card metadata found"
  end
  puts

  # Content analysis
  puts "CONTENT ANALYSIS"
  puts "-" * 80
  puts "Headers found:   #{seo[:header_count]}"
  puts "Links found:     #{seo[:link_count]}"
  puts "Images found:    #{seo[:image_count]}"
  puts

  # Preview of converted markdown
  puts "MARKDOWN OUTPUT PREVIEW"
  puts "-" * 80
  lines = seo[:markdown].split("\n")
  preview = lines.take(15).join("\n")
  puts preview
  puts "... (#{lines.length - 15} more lines)" if lines.length > 15
  puts
end

main
