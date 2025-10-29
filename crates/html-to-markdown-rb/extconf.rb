# frozen_string_literal: true

require 'mkmf'
require 'rb_sys/mkmf'
require 'rbconfig'

def append_bindgen_args(key, args)
  existing = ENV[key].to_s.split(/\s+/)
  ENV[key] = (existing + args).uniq.join(' ') unless args.empty?
end

if RbConfig::CONFIG['host_os'] =~ /mswin|mingw/
  devkit = ENV['RI_DEVKIT']

  if devkit && !devkit.empty?
    include_paths = [
      File.join(devkit, 'ucrt64', 'include'),
      File.join(devkit, 'ucrt64', 'x86_64-w64-mingw32', 'include')
    ].select { |path| Dir.exist?(path) }

    unless include_paths.empty?
      include_args = include_paths.map { |path| "-I#{path.tr('\\', '/')}" }
      append_bindgen_args('BINDGEN_EXTRA_CLANG_ARGS_x86_64-pc-windows-gnu', include_args)
      append_bindgen_args('BINDGEN_EXTRA_CLANG_ARGS_x86_64_pc_windows_gnu', include_args)
      append_bindgen_args('BINDGEN_EXTRA_CLANG_ARGS_x86_64-pc-windows-msvc', include_args)
      append_bindgen_args('BINDGEN_EXTRA_CLANG_ARGS_x86_64_pc_windows_msvc', include_args)
      append_bindgen_args('BINDGEN_EXTRA_CLANG_ARGS', include_args)
    end
  end
end

default_profile = ENV.fetch('CARGO_PROFILE', 'release')

create_rust_makefile('html_to_markdown_rb') do |config|
  config.profile = default_profile.to_sym
end
