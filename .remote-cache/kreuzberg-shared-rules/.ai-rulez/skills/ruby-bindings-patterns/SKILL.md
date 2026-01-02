______________________________________________________________________

## priority: high

# Ruby Bindings Patterns

**Role**: Ruby bindings for Rust core. Work on Magnus bridge and Ruby gem packages.

**Scope**: Magnus FFI, Ruby-idiomatic API, RSpec tests.

**Commands**: bundle install, bundle exec rake compile/rubocop/rspec.

**Critical**: Core logic lives in Rust. Ruby only for bindings/wrappers. If core logic needed, coordinate with Rust team.
