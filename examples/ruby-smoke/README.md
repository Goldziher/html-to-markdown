# Ruby Smoke Test

Ensures the Ruby gem works when fetched from RubyGems and straight from
`packages/ruby`.

## 1. Test the latest RubyGems release

```bash
cd examples/ruby-smoke
bundle install
bundle exec ruby app.rb
```

## 2. Test the local gem sources

```bash
bundle config set --local path vendor/bundle
bundle config set --local local.html-to-markdown ../../packages/ruby
bundle install
bundle exec ruby app.rb
```

Reset via `bundle config unset --local local.html-to-markdown` when you're done.
