# frozen_string_literal: true

require_relative 'lib/html_to_markdown/version'

readme_path = File.expand_path('README.md', __dir__)
readme_body = File.read(readme_path, encoding: 'UTF-8')
repo_root = File.expand_path('..', __dir__)
crate_prefix = 'crates/html-to-markdown-rb/'
git_cmd = %(git -C "#{repo_root}" ls-files -z #{crate_prefix})
git_files =
  `#{git_cmd}`.split("\x0")
              .select { |path| path.start_with?(crate_prefix) }
              .map { |path| path.delete_prefix(crate_prefix) }
fallback_files = Dir.chdir(__dir__) do
  Dir.glob(
    %w[
      Cargo.toml
      Cargo.lock
      README.md
      extconf.rb
      exe/*
      lib/**/*.rb
      lib/bin/*
      src/**/*.rs
      spec/**/*.rb
    ]
  )
end
files = git_files.empty? ? fallback_files : git_files

Gem::Specification.new do |spec|
  spec.name          = 'html-to-markdown'
  spec.version       = HtmlToMarkdown::VERSION
  spec.authors       = ["Na'aman Hirschfeld"]
  spec.email         = ['nhirschfeld@gmail.com']

  spec.summary       = 'Blazing-fast HTML to Markdown conversion for Ruby, powered by Rust.'
  spec.description   = readme_body
  spec.homepage      = 'https://github.com/Goldziher/html-to-markdown'
  spec.license       = 'MIT'

  spec.required_ruby_version = Gem::Requirement.new('>= 3.2')

  spec.bindir = 'exe'
  spec.executables = ['html-to-markdown']
  spec.require_paths = ['lib']

  spec.files = files
  spec.extra_rdoc_files = ['README.md']

  spec.extensions = ['extconf.rb']

  spec.add_dependency 'rb_sys', '>= 0.9', '< 1.0'
  spec.metadata['rubygems_mfa_required'] = 'true'
  spec.metadata['homepage_uri'] = 'https://github.com/Goldziher/html-to-markdown'
  spec.metadata['source_code_uri'] = 'https://github.com/Goldziher/html-to-markdown'
  spec.metadata['bug_tracker_uri'] = 'https://github.com/Goldziher/html-to-markdown/issues'
  spec.metadata['changelog_uri'] = 'https://github.com/Goldziher/html-to-markdown/releases'
  spec.metadata['documentation_uri'] = 'https://github.com/Goldziher/html-to-markdown/blob/main/crates/html-to-markdown-rb/README.md'
end
