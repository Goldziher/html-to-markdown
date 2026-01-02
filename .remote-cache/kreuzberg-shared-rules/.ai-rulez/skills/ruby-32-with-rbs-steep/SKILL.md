______________________________________________________________________

## priority: high

# Ruby 3.2+ with RBS & Steep

**Ruby 3.2+ · RBS type definitions · Steep · rbenv · RSpec · Rubocop**

- Ruby 3.2+ with .ruby-version file; rbenv for version management
- RBS files in sig/ directory parallel to source: lib/foo.rb → sig/foo.rbs
- Steep for type checking; avoid Any types, use union and optional types explicitly
- RSpec for testing: describe/context/it blocks, 80%+ coverage, function-like tests
- Rubocop with auto-fix: line length ≤120, prefer &:method_name blocks
- Naming: PascalCase (classes), snake_case (methods), SCREAMING_SNAKE_CASE (constants)
- Code quality: methods \<10 lines, guard clauses, modules for mixins, Dry::Struct for value objects
- CI: rubocop --format progress, steep check, rspec with simplecov
