# frozen_string_literal: true

require 'mkmf'
require 'rb_sys/mkmf'
require 'rbconfig'
require 'pathname'

if RbConfig::CONFIG['host_os'] =~ /mswin|mingw/
  devkit = ENV.fetch('RI_DEVKIT', nil)
  prefix = ENV['MSYSTEM_PREFIX'] || '/ucrt64'

  if devkit
    sysroot = "#{devkit}#{prefix}".tr('\\\\', '/')
    extra_args = [
      '--target=x86_64-pc-windows-gnu',
      "--sysroot=#{sysroot}"
    ]

    existing = ENV['BINDGEN_EXTRA_CLANG_ARGS'].to_s.split(/\s+/)
    ENV['BINDGEN_EXTRA_CLANG_ARGS'] = (existing + extra_args).uniq.join(' ')
  end
end

default_profile = ENV.fetch('CARGO_PROFILE', 'release')

create_rust_makefile('html_to_markdown_rb') do |config|
  config.profile = default_profile.to_sym
  features_env = ENV.fetch('HTML_TO_MARKDOWN_CARGO_FEATURES', '')
  features = features_env.split(',').map(&:strip).reject(&:empty?)
  config.features = features unless features.empty?

  native_dir = File.expand_path('native', __dir__)
  relative_native =
    begin
      Pathname.new(native_dir).relative_path_from(Pathname.new(__dir__)).to_s
    rescue ArgumentError
      native_dir
    end

  config.ext_dir = relative_native
end
