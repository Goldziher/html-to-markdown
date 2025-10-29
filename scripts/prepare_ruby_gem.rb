#!/usr/bin/env ruby
# frozen_string_literal: true

require "fileutils"
require "pathname"

root = Pathname.new(__dir__).parent

puts "Building html-to-markdown CLI binary..."
cmd = ["cargo", "build", "--release", "--package", "html-to-markdown-cli"]
unless system(*cmd)
  abort "Failed to build html-to-markdown CLI"
end

binary_name = Gem.win_platform? ? "html-to-markdown.exe" : "html-to-markdown"
source = root.join("target", "release", binary_name)
abort "CLI binary not found at #{source}" unless source.file?

bin_dir = root.join("crates", "html-to-markdown-rb", "lib", "bin")
FileUtils.mkdir_p(bin_dir)

plain_binary = bin_dir.join("html-to-markdown")
windows_binary = bin_dir.join("html-to-markdown.exe")

[plain_binary, windows_binary].each do |path|
  next unless path.exist?

  FileUtils.rm_f(path)
end

dest = bin_dir.join(binary_name)
FileUtils.cp(source, dest)
FileUtils.chmod(0o755, dest) unless Gem.win_platform?

puts "Copied CLI binary to #{dest}"
