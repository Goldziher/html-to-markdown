# frozen_string_literal: true

require_relative 'lib/html_to_markdown/version'

Gem::Specification.new do |spec|
  spec.name          = 'html-to-markdown'
  spec.version       = HtmlToMarkdown::VERSION
  spec.authors       = ["Na'aman Hirschfeld"]
  spec.email         = ['nhirschfeld@gmail.com']

  spec.summary       = 'Ruby bindings for the html-to-markdown Rust library'
  spec.description   = 'High-performance HTML to Markdown conversion from Ruby using Magnus and rb-sys.'
  spec.homepage      = 'https://github.com/Goldziher/html-to-markdown'
  spec.license       = 'MIT'

  spec.required_ruby_version = Gem::Requirement.new('>= 3.2')

  spec.bindir = 'exe'
  spec.executables = ['html-to-markdown']
  spec.require_paths = ['lib']

  spec.files = Dir.chdir(__dir__) do
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

  spec.extensions = ['extconf.rb']

  spec.add_dependency 'rb_sys', '>= 0.9', '< 1.0'
  spec.metadata['rubygems_mfa_required'] = 'true'
  spec.metadata['homepage_uri'] = 'https://github.com/Goldziher/html-to-markdown'
  spec.metadata['source_code_uri'] = 'https://github.com/Goldziher/html-to-markdown'
  spec.metadata['bug_tracker_uri'] = 'https://github.com/Goldziher/html-to-markdown/issues'
  spec.metadata['changelog_uri'] = 'https://github.com/Goldziher/html-to-markdown/releases'
  spec.metadata['documentation_uri'] = 'https://github.com/Goldziher/html-to-markdown/blob/main/README.md'
end
