---
priority: medium
---

# Ruby 3.2+ - Magnus Native Bindings with RBS

**Ruby 3.2+ · Magnus native bindings · RBS type definitions · Steep · RSpec · 80%+ coverage**

- Ruby 3.2+ with .ruby-version file; rbenv for version management
- Magnus bindings expose Rust API cleanly; minimal Ruby wrapper logic
- RBS files in packages/ruby/sig/ parallel to source (lib/foo.rb → sig/foo.rbs)
- Steep for type checking; avoid Any types, use union and optional types explicitly
- RSpec in packages/ruby/spec/; describe/context/it blocks, 80%+ coverage
- Rubocop with auto-fix: line length ≤120, prefer &:method_name blocks
- Distribution: Ruby gem via bundle exec rake package
- Never: business logic duplication; binding code defers to Rust
- Use Haiku 4.5 for Ruby binding engineering and RBS issues
