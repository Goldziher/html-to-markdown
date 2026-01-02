______________________________________________________________________

## priority: high

# Elixir 1.14+ - Rustler NIF Bindings

**Elixir 1.14+ · OTP 25+ · Rustler NIFs · Pattern matching · ExUnit · ExDoc · 80%+ coverage**

- Target Elixir 1.14+ with OTP 25+; use match/case and newer Elixir features
- Rustler NIF bindings in crates/ (crates/\*-elixir) provide safe FFI to Rust code
- Mix project structure in packages/elixir; use Nif.module/1 for clean API exposure
- Functional API patterns: leverage Elixir's pattern matching, immutability, pipe operators
- Error handling via standard Elixir tuples: {:ok, result} | {:error, reason}
- Testing: ExUnit in packages/elixir/test/; assert_raise for error cases, 80%+ coverage
- Credo for static analysis; avoid unused variables, simplify function bodies
- ExDoc for documentation; @doc/@spec for all public functions, typespecs required
- Never: mutable state in NIFs, long-running Elixir threads in Rust, unhandled panics
- Use Haiku 4.5 for elixir-bindings-engineer tasks and Rustler integration issues

## Rustler NIF Crate Structure

Rustler NIF crates follow the pattern `crates/{module_name}-elixir`:

```
crates/my-module-elixir/
  ├── Cargo.toml          # NIF crate metadata, rustler dependency
  ├── src/
  │   └── lib.rs          # NIF module with #[nif] macros
  └── mix.exs             # Mix configuration for native compilation
```

**Requirements:**

- Rustler dependency pinned to stable version
- lib.rs exports #[module] with all NIF functions
- Each NIF function returns Result\<T, String> with clear error messages
- No panics; convert Rust errors to Elixir error tuples
- All NIF functions must have @doc comments in wrapper module

## Mix Project Setup

Elixir Mix projects consuming NIFs:

```
packages/elixir/
  ├── mix.exs             # Application, deps, :rustler compiler
  ├── lib/
  │   ├── {app}_name.ex        # Main application module
  │   └── {app_name}/
  │       └── nif.ex      # NIF wrapper with typespecs
  ├── test/
  │   ├── test_helper.exs
  │   └── {app_name}_test.exs
  └── .credo.exs          # Credo configuration
```

**Requirements:**

- Credo configuration in .credo.exs; run `mix credo` before commits
- All public functions must have @spec typespecs
- @doc required for all public functions with usage examples
- test/ directory contains all test files (no nested subdirectories)
- ExDoc configured for mix docs generation

## API Design & Pattern Matching

Functional Elixir APIs leverage pattern matching:

**Good:**

```elixir
defmodule MyModule do
  @spec process(term()) :: {:ok, any()} | {:error, String.t()}
  def process(input) do
    with {:ok, value} <- validate(input),
         {:ok, result} <- nif_process(value) do
      {:ok, result}
    else
      {:error, reason} -> {:error, reason}
    end
  end

  defp nif_process(value), do: NIF.process(value)
end
```

**Anti-patterns:**

- Returning raw tuples without proper typespecs
- Silent failures (returning :ok without validating input)
- Raising exceptions instead of returning {:error, reason}
- Mixing error handling styles (some exceptions, some tuples)

## Error Handling

All NIF calls must return either {:ok, value} or {:error, reason}:

```elixir
defmodule MyModule.NIF do
  @doc "Call Rust NIF function"
  @spec compute(term()) :: {:ok, term()} | {:error, String.t()}
  def compute(input) do
    case nif_compute_impl(input) do
      {:ok, result} -> {:ok, result}
      {:error, msg} -> {:error, "Computation failed: #{msg}"}
    end
  end

  defnif nif_compute_impl(input)
end
```

**Requirements:**

- Never raise exceptions from NIFs; return error tuples
- Error messages must be descriptive and actionable
- Use pattern matching in caller code: with {:ok, x} \<- nif_call(y)
- Validate Elixir inputs before passing to Rust
- Document error cases in @doc strings

## Testing with ExUnit

ExUnit tests validate bindings and error cases:

```elixir
defmodule MyModuleTest do
  use ExUnit.Case
  doctest MyModule

  describe "process/1" do
    test "returns {:ok, result} on success" do
      assert {:ok, result} = MyModule.process(valid_input())
      assert result != nil
    end

    test "returns {:error, reason} on invalid input" do
      assert {:error, _reason} = MyModule.process(invalid_input())
    end

    test "raises if NIF is unavailable" do
      assert_raise UndefinedFunctionError, fn ->
        MyModule.missing_nif(1)
      end
    end
  end

  defp valid_input, do: %{"key" => "value"}
  defp invalid_input, do: nil
end
```

**Requirements:**

- Test both success and error paths
- 80%+ code coverage measured with ExCoveralls
- Use describe blocks to organize tests
- Test error messages for clarity and actionability
- Document fixtures and helper functions

## Credo Linting

Credo enforces code quality:

```
mix credo --strict
```

**Configuration in .credo.exs:**

- Line length limit: 120 characters
- Avoid unused variables: {Credo.Check.Warning.Unused, []}
- Simplify function definitions
- Prefer descriptive names over abbreviations
- No empty function bodies without reason

**Requirements:**

- Run `mix credo` before pushing; fix all warnings
- Never ignore checks with # credo:disable-for-next-line
- Unused imports removed automatically: `mix credo --fix`

## ExDoc Documentation

All public functions must be documented for mix docs:

```elixir
defmodule MyModule do
  @moduledoc """
  Module description explaining purpose and usage.

  ## Examples

      iex> MyModule.process(:input)
      {:ok, :result}
  """

  @doc """
  Processes the given input using Rust NIF.

  ## Arguments
    - `input` - Term to process; must be serializable

  ## Returns
    - `{:ok, result}` on success
    - `{:error, reason}` on failure

  ## Examples
      iex> MyModule.process("hello")
      {:ok, "HELLO"}

      iex> MyModule.process(nil)
      {:error, "Invalid input"}
  """
  @spec process(term()) :: {:ok, any()} | {:error, String.t()}
  def process(input) do
    NIF.process(input)
  end
end
```

**Requirements:**

- @moduledoc on every module explaining its purpose
- @doc on all public functions with examples
- @spec on all public functions with proper types
- iex examples in docs must be runnable (mix test --doc)
- Links to related modules using Markdown [Module](%60Module%60)

## Build & Compilation

Rustler compilation via Mix:

```elixir
# mix.exs
defp compilers() do
  [:rustler] ++ Mix.compilers()
end

defp rustler_crates() do
  [
    my_module_elixir: [
      path: "../crates/my-module-elixir",
      cargo_toml_dir: :auto
    ]
  ]
end
```

**Requirements:**

- Rustler compiler configured before other compilers
- Cargo.toml path points to correct crate directory
- Build runs: `mix compile` (automatic)
- Clean builds: `mix clean && mix compile`
- Version mismatch errors caught at compile time

## Dependency Management

Mix dependencies pinned in mix.lock:

```elixir
# mix.exs
defp deps do
  [
    {:rustler, "~> 0.34"},
    {:ex_doc, "~> 0.34", only: :dev},
    {:credo, "~> 1.7", only: [:dev, :test]},
    {:excoveralls, "~> 0.18", only: :test}
  ]
end
```

**Requirements:**

- All dependencies pinned with mix.lock committed
- Use tilde constraints: ~> X.Y for stability
- Dev/test dependencies in only: [:dev, :test]
- Periodically update: mix deps.update --all
- Check for security updates: mix deps.audit

## Anti-Patterns to Avoid

**Never do this:**

1. Raise exceptions in NIF wrapper instead of returning error tuples
1. Pass mutable state across Elixir/Rust boundary
1. Perform I/O in NIF functions (blocks scheduler)
1. Skip error handling with "it won't fail"
1. Duplicate logic in Elixir and Rust (single source of truth in Rust)
1. Use Any types in typespecs; be explicit
1. Test only the happy path; test error cases thoroughly

**Good alternatives:**

1. Always return {:ok, value} | {:error, reason}
1. Use Elixir processes for shared state
1. Offload I/O to Elixir; NIFs compute only
1. Document all failure modes in @doc
1. Keep Elixir code thin; business logic in Rust
1. Use union types and specific types in specs
1. Test error conditions, edge cases, and invalid inputs

## Agent References

For Elixir-specific binding engineering:

- **elixir-bindings-engineer**: Architecture, API design, pattern matching, Rustler integration
- **Use for**: NIF wrapper design, error handling strategies, performance optimization
- **Reference in commits**: Consult elixir-bindings-engineer for binding architecture decisions
