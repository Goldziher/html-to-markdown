# frozen_string_literal: true

require_relative 'lib/html_to_markdown/version'

readme_path = File.expand_path('README.md', __dir__)
readme_body = File.read(readme_path, encoding: 'UTF-8')

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

  spec.files = `git ls-files -z`.split("\x0")
  spec.extra_rdoc_files = ['README.md']

  spec.extensions = ['extconf.rb']

  spec.add_dependency 'rb_sys', '>= 0.9', '< 1.0'
  spec.metadata['rubygems_mfa_required'] = 'true'
  spec.metadata['homepage_uri'] = 'https://github.com/Goldziher/html-to-markdown'
  spec.metadata['source_code_uri'] = 'https://github.com/Goldziher/html-to-markdown'
  spec.metadata['bug_tracker_uri'] = 'https://github.com/Goldziher/html-to-markdown/issues'
  spec.metadata['changelog_uri'] = 'https://github.com/Goldziher/html-to-markdown/releases'
  spec.metadata['documentation_uri'] = 'https://github.com/Goldziher/html-to-markdown/blob/main/README.md'
end
