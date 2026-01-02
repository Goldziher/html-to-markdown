______________________________________________________________________

## priority: critical

# Elixir 1.14+ with ExDoc & Credo

**Elixir 1.14+ · Functional-first · ExDoc · ExUnit · Rustler NIF**

- Target Elixir 1.14+ with .tool-versions file; use ASDF for version management
- Module structure: CamelCase for modules (e.g., Kreuzberg.Native), snake_case for functions/variables
- ExDoc for documentation: @moduledoc and @doc tags on all public APIs with examples
- ExUnit for testing: describe/test blocks, 80%+ test coverage, property-based testing with StreamData
- Credo for linting: strict mode enabled, no ignored warnings
- Type specifications: @spec, @type, @callback annotations on all public functions
- Guard clauses and pattern matching for control flow (avoid if/else chains)
- OTP principles: Supervision trees, GenServer for state, Agent for simple storage
- Erlang/OTP interoperability: understand BEAM VM concepts (processes, mailboxes, schedulers)
- Result tuples: {:ok, value} and {:error, reason} patterns (NEVER exceptions for control flow)
- Immutable data structures: pipelines with |> operator for transformations
- Pure functions: side effects isolated to GenServer/Agent/NIFs
- Naming: PascalCase (modules), snake_case (functions), SCREAMING_SNAKE_CASE (module attributes)
- Code quality: Functions concise (\<20 lines), guard clauses, modules for logical organization
- Rustler NIF: Schedule CPU-intensive work on dirty schedulers (schedule = "DirtyCpu")
- Binary handling: Use Erlang binaries efficiently, avoid unnecessary copying
- CI: mix credo --strict, mix test with coverage, mix format --check-formatted, mix docs
