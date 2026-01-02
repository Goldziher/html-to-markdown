______________________________________________________________________

## priority: high

# Fixture-Driven Testing Strategy

## Shared Test Fixtures Across Language Bindings

- **Single source of truth**: Fixtures defined once in Rust, exposed to all language bindings
- **API parity validation**: Identical behavior across languages guaranteed by fixtures
- **Fixture generation**: Rust generates JSON/YAML fixtures from canonical implementation
- **Language-specific consumption**: Each binding consumes same fixtures via fixture format (JSON, YAML, MessagePack)

Example directory structure:

```
project/
├── crates/
│   ├── core/
│   │   ├── src/
│   │   ├── tests/
│   │   └── fixtures/
│   │       ├── basic.json
│   │       ├── edge_cases.json
│   │       └── generator.rs  # Generates fixtures from canonical impl
│   └── ffi/
│       └── src/lib.rs
├── bindings/
│   ├── python/
│   │   ├── tests/
│   │   │   ├── conftest.py  # Load fixtures
│   │   │   └── test_*.py
│   │   └── fixtures -> ../../crates/core/fixtures  # Symlink
│   ├── node/
│   │   ├── __tests__/
│   │   └── fixtures -> ../../crates/core/fixtures
│   ├── ruby/
│   │   ├── spec/
│   │   └── fixtures -> ../../crates/core/fixtures
│   ├── java/
│   │   ├── src/test/
│   │   └── resources/fixtures -> ../../crates/core/fixtures
│   └── go/
│       ├── *_test.go
│       └── testdata/fixtures -> ../../crates/core/fixtures
```

## Fixture Formats

### JSON Format (Primary)

```json
{
  "version": 1,
  "generated_at": "2025-01-15T10:30:00Z",
  "test_cases": [
    {
      "name": "basic_case",
      "input": {
        "data": "sample input value"
      },
      "expected_output": {
        "result": "expected output value"
      },
      "description": "Basic processing case"
    },
    {
      "name": "advanced_case",
      "input": {
        "data": "complex data structure"
      },
      "expected_output": {
        "result": "transformed result"
      }
    },
    {
      "name": "edge_case_empty_input",
      "input": {
        "data": ""
      },
      "expected_output": {
        "result": ""
      }
    },
    {
      "name": "special_characters",
      "input": {
        "data": "input with special chars & symbols"
      },
      "expected_output": {
        "result": "output with special chars & symbols"
      }
    },
    {
      "name": "error_case_invalid_input",
      "input": {
        "data": "malformed input"
      },
      "expected_error": "validation_error"
    }
  ]
}
```

### YAML Format (Human-Readable)

```yaml
version: 1
generated_at: 2025-01-15T10:30:00Z

test_cases:
  - name: basic_case
    input:
      data: "sample input"
    expected_output:
      result: "expected output"

  - name: advanced_case
    input:
      data: "complex input structure"
    expected_output:
      result: "transformed result"

  - name: special_chars
    input:
      data: "input with special & characters"
    expected_output:
      result: "output with special & characters"

  - name: unicode_handling
    input:
      data: "Hello 世界 🌍"
    expected_output:
      result: "Hello 世界 🌍"
```

## Fixture Generation from Rust

Generate fixtures programmatically from Rust canonical implementation:

```rust
// crates/core/src/lib.rs - Canonical implementation
pub fn process(input: &str) -> Result<String> {
    // Core processing logic
    // ...
}

#[cfg(test)]
mod fixture_generation {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    #[ignore]  // Run manually with: cargo test -- --ignored --nocapture
    fn generate_fixtures() {
        let test_cases = vec![
            ("basic_case", "sample input 1"),
            ("advanced_case", "sample input 2"),
            ("edge_case", "sample input 3"),
            ("empty_input", ""),
        ];

        let mut fixtures = json!({
            "version": 1,
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "test_cases": []
        });

        let cases = fixtures["test_cases"].as_array_mut().unwrap();

        for (name, input) in test_cases {
            match process(input) {
                Ok(output) => {
                    cases.push(json!({
                        "name": name,
                        "input": { "data": input },
                        "expected_output": { "result": output }
                    }));
                }
                Err(e) => {
                    cases.push(json!({
                        "name": name,
                        "input": { "data": input },
                        "expected_error": e.to_string()
                    }));
                }
            }
        }

        let output_path = "crates/core/fixtures/canonical.json";
        fs::write(
            output_path,
            serde_json::to_string_pretty(&fixtures).unwrap()
        ).unwrap();

        println!("Generated fixtures to {}", output_path);
    }
}
```

Usage:

```bash
cargo test --package core -- --ignored fixture_generation --nocapture
```

## Python Fixture Consumption

```python
import json
import pytest
from pathlib import Path

@pytest.fixture(scope="session")
def fixtures():
    """Load shared test fixtures from JSON"""
    fixture_path = Path(__file__).parent.parent.parent / "crates" / "core" / "fixtures" / "canonical.json"
    with open(fixture_path) as f:
        return json.load(f)

@pytest.mark.parametrize("test_case", fixtures()["test_cases"], ids=lambda tc: tc["name"])
def test_process(test_case):
    """Test against shared fixtures"""
    from my_module import process

    input_data = test_case["input"]["data"]

    if "expected_error" in test_case:
        with pytest.raises(Exception) as exc_info:
            process(input_data)
        assert test_case["expected_error"] in str(exc_info.value)
    else:
        expected_result = test_case["expected_output"]["result"]
        actual_result = process(input_data)
        assert actual_result == expected_result

    # Display test metadata
    if "description" in test_case:
        print(f"\n{test_case['description']}")
```

## TypeScript/Node.js Fixture Consumption

```typescript
import * as fs from "fs";
import { process } from "../src";

interface TestCase {
  name: string;
  input: { data: string };
  expected_output?: { result: string };
  expected_error?: string;
  description?: string;
}

const fixtures: TestCase[] = JSON.parse(
  fs.readFileSync(
    "../../crates/core/fixtures/canonical.json",
    "utf-8"
  )
).test_cases;

describe("Fixture-driven tests", () => {
  fixtures.forEach((testCase) => {
    it(`${testCase.name}: ${testCase.description || ""}`, async () => {
      if (testCase.expected_error) {
        expect(() => process(testCase.input.data)).toThrow(
          testCase.expected_error
        );
      } else {
        const result = process(testCase.input.data);
        expect(result).toBe(testCase.expected_output!.result);
      }
    });
  });
});
```

## Ruby Fixture Consumption

```ruby
require 'json'
require 'rspec'
require 'my_module'

fixtures = JSON.parse(
  File.read(File.expand_path('../../../../crates/core/fixtures/canonical.json', __FILE__))
)

describe MyModule do
  fixtures['test_cases'].each do |test_case|
    context test_case['name'] do
      it test_case['description'] || 'processes correctly' do
        input_data = test_case['input']['data']

        if test_case['expected_error']
          expect { MyModule.process(input_data) }.to raise_error
        else
          expected_result = test_case['expected_output']['result']
          actual_result = MyModule.process(input_data)
          expect(actual_result).to eq(expected_result)
        end
      end
    end
  end
end
```

## Java Fixture Consumption

```java
import com.google.common.io.Resources;
import com.google.gson.Gson;
import com.google.gson.JsonObject;
import com.google.gson.JsonArray;
import static org.junit.jupiter.api.Assertions.*;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.MethodSource;

public class ProcessFixtureTest {

    static class TestCase {
        String name;
        Input input;
        Output expected_output;
        String expected_error;
    }

    static class Input {
        String data;
    }

    static class Output {
        String result;
    }

    static Stream<TestCase> loadFixtures() throws Exception {
        String fixturesJson = Resources.toString(
            Resources.getResource("fixtures/canonical.json"),
            StandardCharsets.UTF_8
        );

        Gson gson = new Gson();
        JsonObject root = gson.fromJson(fixturesJson, JsonObject.class);
        JsonArray testCases = root.getAsJsonArray("test_cases");

        return StreamSupport.stream(testCases.spliterator(), false)
            .map(elem -> gson.fromJson(elem, TestCase.class));
    }

    @ParameterizedTest(name = "{0}")
    @MethodSource("loadFixtures")
    void testProcess(TestCase testCase) {
        if (testCase.expected_error != null) {
            assertThrows(Exception.class, () ->
                MyModule.process(testCase.input.data)
            );
        } else {
            String result = MyModule.process(testCase.input.data);
            assertEquals(testCase.expected_output.result, result);
        }
    }
}
```

## Snapshot Testing

Use snapshot testing for complex outputs (diffs shown in PRs):

```rust
// Rust with insta crate
use insta::assert_snapshot;

#[test]
fn test_complex_input_snapshot() {
    let input = r#"
        Complex multiline
        structured input
        with nested data
    "#;

    let result = process(input).unwrap();
    assert_snapshot!(result);
}
```

```python
# Python with pytest-snapshot or syrupy
def test_complex_input_snapshot(snapshot):
    input_data = """
        Complex multiline
        structured input
        with nested data
    """
    result = process(input_data)
    assert result == snapshot
```

```typescript
// TypeScript with jest snapshots
test("complex input snapshot", () => {
  const input = `
    Complex multiline
    structured input
    with nested data
  `;

  expect(process(input)).toMatchSnapshot();
});
```

## Cross-Language Test Consistency

Ensure identical behavior across bindings:

```bash
#!/bin/bash
# test_parity.sh - Run fixture tests across all bindings and compare

set -e

FIXTURE_FILE="crates/core/fixtures/canonical.json"

echo "Testing Rust (canonical)..."
cargo test --package core --test fixture_driven

echo "Testing Python..."
cd bindings/python && python -m pytest tests/test_fixtures.py -v && cd ../..

echo "Testing Node.js..."
cd bindings/node && npm test -- test/fixtures.test.ts && cd ../..

echo "Testing Ruby..."
cd bindings/ruby && bundle exec rspec spec/fixtures_spec.rb && cd ../..

echo "Testing Java..."
cd bindings/java && ./gradlew test --tests "*FixtureTest" && cd ../..

echo "All binding tests passed with consistent behavior!"
```

Run in CI:

```yaml
- name: Cross-language fixture parity test
  run: bash scripts/test_parity.sh
```

## Anti-Patterns

- **Language-specific fixtures**: Define once in Rust, reuse everywhere
- **Hardcoded test data**: Use fixture files for maintainability
- **No error case testing**: Include `expected_error` cases in fixtures
- **Incomplete fixtures**: Cover edge cases, unicode, special characters
- **Manual fixture updates**: Generate from canonical implementation automatically
- **No fixture versioning**: Include version field to track breaking changes
- **Test duplication**: Write fixture consumer once per language, parameterize tests
- **Snapshot conflicts in PR**: Use deterministic snapshot naming
- **No fixture documentation**: Include `description` field explaining intent
