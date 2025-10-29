# frozen_string_literal: true

require 'mkmf'
require 'rb_sys/mkmf'
require 'rbconfig'
require 'fileutils'

def append_bindgen_args(key, args)
  existing = ENV[key].to_s.split(/\s+/)
  merged = (existing + args).reject(&:empty?).uniq
  ENV[key] = merged.join(' ') unless merged.empty?
  merged
end

def append_bindgen_include(path)
  normalized = path.tr('\\', '/')
  {
    'BINDGEN_EXTRA_CLANG_ARGS_x86_64-pc-windows-gnu' => ["-I#{normalized}"],
    'BINDGEN_EXTRA_CLANG_ARGS_x86_64_pc_windows_gnu' => ["-I#{normalized}"],
    'BINDGEN_EXTRA_CLANG_ARGS_x86_64-pc-windows-msvc' => ["-I#{normalized}"],
    'BINDGEN_EXTRA_CLANG_ARGS_x86_64_pc_windows_msvc' => ["-I#{normalized}"],
    'BINDGEN_EXTRA_CLANG_ARGS_X86_64_PC_WINDOWS_GNU' => ["-I#{normalized}"],
    'BINDGEN_EXTRA_CLANG_ARGS_X86_64_PC_WINDOWS_MSVC' => ["-I#{normalized}"],
    'BINDGEN_EXTRA_CLANG_ARGS' => ["-I#{normalized}"]
  }
end

bindgen_keys = %w[
  BINDGEN_EXTRA_CLANG_ARGS
  BINDGEN_EXTRA_CLANG_ARGS_x86_64-pc-windows-gnu
  BINDGEN_EXTRA_CLANG_ARGS_x86_64_pc_windows_gnu
  BINDGEN_EXTRA_CLANG_ARGS_x86_64-pc-windows-msvc
  BINDGEN_EXTRA_CLANG_ARGS_x86_64_pc_windows_msvc
  BINDGEN_EXTRA_CLANG_ARGS_X86_64_PC_WINDOWS_GNU
  BINDGEN_EXTRA_CLANG_ARGS_X86_64_PC_WINDOWS_MSVC
]

bindgen_env_overrides = Hash.new { |h, k| h[k] = [] }
bindgen_keys.each { |key| bindgen_env_overrides[key] }
cpath_entries = []
cxxpath_entries = []
rbconfig_overrides = Hash.new { |h, k| h[k] = [] }

if RbConfig::CONFIG['host_os'] =~ /mswin|mingw/
  devkit = ENV['RI_DEVKIT']

  if devkit && !devkit.empty?
potential_roots = [
  File.join(devkit, 'ucrt64', 'include'),
  File.join(devkit, 'ucrt64', 'x86_64-w64-mingw32', 'include'),
  File.join(devkit, 'ucrt64', 'include', 'ucrt'),
  File.join(devkit, 'ucrt64', 'include', 'c++'),
  File.join(devkit, 'ucrt64', 'include', 'c++', 'x86_64-w64-mingw32')
]
local_include = File.expand_path('include', __dir__)
potential_roots << local_include if Dir.exist?(local_include)
local_strings_header = File.join(local_include, 'strings.h').tr('\\', '/') if Dir.exist?(local_include)
if local_strings_header && File.exist?(local_strings_header)
  fallback_targets = [
    File.join(devkit, 'ucrt64', 'include', 'strings.h'),
    File.join(devkit, 'ucrt64', 'x86_64-w64-mingw32', 'include', 'strings.h')
  ]

  fallback_targets.each do |target|
    next if File.exist?(target)

    begin
      FileUtils.mkdir_p(File.dirname(target))
      FileUtils.cp(local_strings_header, target)
      warn "Installed fallback strings.h at #{target}"
    rescue StandardError => e
      warn "Failed to install fallback strings.h at #{target}: #{e.message}"
    end
  end

  bindgen_keys.each do |key|
    bindgen_env_overrides[key] << '-include'
    bindgen_env_overrides[key] << local_strings_header
  end
end

    potential_roots.each do |path|
      next unless Dir.exist?(path)

      warn "bindgen include path detected: #{path}"
      normalized = path.tr('\\', '/')
      cpath_entries << normalized
      cxxpath_entries << normalized
      %w[CPPFLAGS CFLAGS CXXFLAGS].each do |key|
        rbconfig_overrides[key] << "-I#{normalized}"
      end

      append_bindgen_include(path).each do |key, values|
        bindgen_env_overrides[key].concat(values)
      end
    end

    sysroot = File.join(devkit, 'ucrt64')
    if Dir.exist?(sysroot)
      sysroot_arg = "--sysroot=#{sysroot.tr('\\', '/')}"
      bindgen_keys.each do |key|
        bindgen_env_overrides[key] << sysroot_arg
      end

      include_dir = File.join(sysroot, 'include').tr('\\', '/')
      %w[CPPFLAGS CFLAGS CXXFLAGS].each do |key|
        rbconfig_overrides[key] << "-I#{include_dir}"
      end
    end
  end
end

default_profile = ENV.fetch('CARGO_PROFILE', 'release')

create_rust_makefile('html_to_markdown_rb') do |config|
  config.profile = default_profile.to_sym
  warn "rb-sys config class=#{config.class}"
  debug_methods = config.methods.grep(/cflags|cargo|env|push|add/i).sort
  warn "rb-sys config methods=#{debug_methods}"

  bindgen_env_overrides.each do |key, values|
    next if values.empty?

    existing = (config.env[key].to_s.split(/\s+/) + ENV[key].to_s.split(/\s+/)).reject(&:empty?)
    combined = (existing + values).uniq
    next if combined.empty?

    config.env[key] = combined.join(' ')
    ENV[key] = combined.join(' ')
    warn "bindgen env #{key}=#{combined.join(' ')}"
  end

  unless cpath_entries.empty?
    combined_cpath = (ENV['CPATH'].to_s.split(File::PATH_SEPARATOR) + cpath_entries).reject(&:empty?).uniq
    value = combined_cpath.join(File::PATH_SEPARATOR)
    config.env['CPATH'] = value
    ENV['CPATH'] = value
    warn "CPATH=#{value}"
  end

  unless cxxpath_entries.empty?
    combined_cxx = (ENV['CPLUS_INCLUDE_PATH'].to_s.split(File::PATH_SEPARATOR) + cxxpath_entries).reject(&:empty?).uniq
    value = combined_cxx.join(File::PATH_SEPARATOR)
    config.env['CPLUS_INCLUDE_PATH'] = value
    ENV['CPLUS_INCLUDE_PATH'] = value
    warn "CPLUS_INCLUDE_PATH=#{value}"
  end

  rbconfig_overrides.each do |key, values|
    next if values.empty?

    existing = ENV["RBCONFIG_#{key}"].to_s.split(/\s+/)
    combined = (existing + values).uniq
    next if combined.empty?

    ENV["RBCONFIG_#{key}"] = combined.join(' ')
    warn "RBCONFIG_#{key}=#{ENV["RBCONFIG_#{key}"]}"
  end
end
