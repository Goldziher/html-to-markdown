# Testing Strategy

## Overview

Testing ensures correctness, safety, and API consistency across the polyglot ecosystem. The strategy enforces high coverage requirements (95% for Rust core, 80% for bindings) with fixture-driven tests validating API parity.

## Coverage Requirements

### Rust Core: 95% Coverage Requirement

```bash
# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage \
  --exclude-files tests/ \
  --timeout 300

# Enforce minimum coverage threshold
cargo tarpaulin --out Stdout --fail-under 95
```

**Coverage Definition:**

- Lines executed: 95% minimum
- Branches covered: 90% minimum
- Excluded: test-only code, examples, benches

**High-Coverage Strategies:**

1. **Unit Tests** (70% of coverage)

   ```rust
   // src/models/request.rs
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_request_validation() {
           let req = Request {
               url: "http://example.com".to_string(),
               method: "GET".to_string(),
               headers: HashMap::new(),
               body: None,
           };
           assert!(req.validate().is_ok());
       }

       #[test]
       fn test_request_validation_invalid_url() {
           let req = Request {
               url: "not a url".to_string(),
               method: "GET".to_string(),
               headers: HashMap::new(),
               body: None,
           };
           assert!(req.validate().is_err());
       }

       #[test]
       fn test_request_validation_invalid_method() {
           let req = Request {
               url: "http://example.com".to_string(),
               method: "INVALID".to_string(),
               headers: HashMap::new(),
               body: None,
           };
           assert!(req.validate().is_err());
       }
   }
   ```

1. **Integration Tests** (20% of coverage)

   ```rust
   // tests/integration_tests.rs
   #[tokio::test]
   async fn test_client_execute_success() {
       let config = Config {
           timeout_ms: 5000,
           max_retries: 3,
       };
       let client = Client::new(config).unwrap();

       let request = Request {
           url: "http://httpbin.org/get".to_string(),
           method: "GET".to_string(),
           headers: HashMap::new(),
           body: None,
       };

       let response = client.execute(request).await;
       assert!(response.is_ok());
       assert_eq!(response.unwrap().status_code, 200);
   }

   #[tokio::test]
   async fn test_client_timeout() {
       let config = Config {
           timeout_ms: 1,  // Very short timeout
           max_retries: 1,
       };
       let client = Client::new(config).unwrap();

       let request = Request {
           url: "http://httpbin.org/delay/10".to_string(),  // 10s delay
           method: "GET".to_string(),
           headers: HashMap::new(),
           body: None,
       };

       let response = client.execute(request).await;
       assert!(matches!(response, Err(Error::Timeout(_))));
   }
   ```

1. **Doc Tests** (5% of coverage)

   ````rust
   /// # Examples
   ///
   /// ```
   /// # use mylib::{Client, Config};
   /// # #[tokio::main]
   /// # async fn main() -> Result<(), mylib::Error> {
   /// let config = Config::default();
   /// let client = Client::new(config)?;
   /// # Ok(())
   /// # }
   /// ```
   pub async fn execute(&self, request: Request) -> Result<Response> {
       // Implementation
   }
   ````

### Language Bindings: 80% Coverage Requirement

```bash
# Python binding coverage
cd bindings/python
pytest --cov=mylib --cov-fail-under=80 --cov-report=html

# TypeScript binding coverage
cd bindings/typescript
npm test -- --coverage --coverageThreshold='{"global":{"lines":80,"statements":80,"functions":80,"branches":80}}'
```

## Fixture-Driven Testing for API Parity

### Shared Test Fixtures

Fixtures are language-neutral test data used to validate API consistency:

```yaml
# tests/fixtures/api-parity.yaml
test_cases:
  - name: "create_client_success"
    description: "Client creation with valid config"
    input:
      config:
        timeout_ms: 5000
        max_retries: 3
    expected:
      status: "success"
      client: "non-null"

  - name: "create_client_invalid_config"
    description: "Client creation with invalid timeout"
    input:
      config:
        timeout_ms: 0
        max_retries: 3
    expected:
      status: "error"
      error_type: "InvalidConfig"
      error_message_contains: "timeout"

  - name: "execute_request_success"
    description: "Execute valid HTTP request"
    input:
      request:
        url: "http://httpbin.org/get"
        method: "GET"
        headers: {}
        body: null
    expected:
      status: "success"
      response:
        status_code: 200

  - name: "execute_request_timeout"
    description: "Request exceeds timeout"
    input:
      config:
        timeout_ms: 1
      request:
        url: "http://httpbin.org/delay/10"
        method: "GET"
    expected:
      status: "error"
      error_type: "Timeout"

  - name: "execute_request_network_error"
    description: "Network unreachable"
    input:
      request:
        url: "http://invalid-domain-that-does-not-exist-xyz.com"
        method: "GET"
    expected:
      status: "error"
      error_type: "NetworkError"
```

### Test Implementation Across Languages

```rust
// tests/api_parity/mod.rs - Rust reference implementation
pub struct TestFixture {
    pub name: String,
    pub input: serde_json::Value,
    pub expected: serde_json::Value,
}

impl TestFixture {
    pub fn load_all() -> Vec<Self> {
        let yaml = std::fs::read_to_string("tests/fixtures/api-parity.yaml")
            .expect("Failed to load fixtures");
        serde_yaml::from_str(&yaml).expect("Failed to parse fixtures")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_parity() {
        for fixture in TestFixture::load_all() {
            // Execute test based on fixture
            // Verify results match expected output
        }
    }
}
```

```python
# bindings/python/tests/test_api_parity.py
import yaml
import pytest
from mylib import Client, Config

@pytest.fixture
def api_fixtures():
    with open('../../tests/fixtures/api-parity.yaml') as f:
        return yaml.safe_load(f)['test_cases']

@pytest.mark.parametrize("fixture", api_fixtures(), ids=lambda f: f['name'])
async def test_api_parity(fixture):
    """Validate API behavior matches Rust core"""
    if fixture['name'] == 'create_client_success':
        config = Config(
            timeout_ms=fixture['input']['config']['timeout_ms'],
            max_retries=fixture['input']['config']['max_retries']
        )
        client = Client(config)
        assert client is not None
```

```typescript
// bindings/typescript/tests/api-parity.test.ts
import * as fs from 'fs';
import * as yaml from 'yaml';
import { Client, Config } from '../dist/index';

const fixtures = yaml.parse(
  fs.readFileSync('../../tests/fixtures/api-parity.yaml', 'utf-8')
).test_cases;

describe('API Parity Tests', () => {
  fixtures.forEach((fixture) => {
    test(`${fixture.name}: ${fixture.description}`, async () => {
      if (fixture.name === 'create_client_success') {
        const config = new Config(
          fixture.input.config.timeout_ms,
          fixture.input.config.max_retries
        );
        const client = new Client(config);
        expect(client).toBeDefined();
      }
    });
  });
});
```

## Test Organization

### Directory Structure

```
tests/
├── fixtures/
│   ├── api-parity.yaml         # Shared test fixtures
│   ├── sample-responses.json   # Mock HTTP responses
│   └── error-cases.yaml        # Error scenario fixtures
├── integration/
│   ├── client_tests.rs         # Client functionality
│   ├── transport_tests.rs      # Transport layer
│   └── cache_tests.rs          # Caching behavior
├── api_parity/
│   ├── mod.rs                  # Fixture loader
│   └── tests.rs                # Parity validation
└── support/
    ├── mock_server.rs          # Mock HTTP server
    ├── test_config.rs          # Test configurations
    └── fixtures.rs             # Fixture helpers

bindings/python/tests/
├── conftest.py                 # Pytest configuration
├── test_api_parity.py          # API parity tests
├── test_binding_types.py       # Type mapping tests
└── test_fixtures.py            # Fixture loading

bindings/typescript/tests/
├── setup.ts                    # Test setup
├── api-parity.test.ts          # API parity tests
└── binding-types.test.ts       # Type mapping tests
```

### Test Categories

````rust
// Unit Tests - Isolated component testing
#[cfg(test)]
mod unit_tests {
    #[test]
    fn test_config_validation() { /* */ }

    #[test]
    fn test_request_parsing() { /* */ }

    #[test]
    fn test_error_conversion() { /* */ }
}

// Integration Tests - Multiple components together
#[tokio::test]
async fn test_client_with_retry() { /* */ }

#[tokio::test]
async fn test_client_with_cache() { /* */ }

// Doc Tests - Inline documentation examples
/// # Examples
/// ```
/// # use mylib::{Client, Config};
/// # #[tokio::main]
/// # async fn main() -> Result<(), mylib::Error> {
/// let config = Config::default();
/// let client = Client::new(config)?;
/// # Ok(())
/// # }
/// ```

// E2E Tests - Full user workflows
#[tokio::test]
async fn test_full_request_lifecycle() {
    // Create client
    // Execute multiple requests
    // Verify results
}
````

## CI Test Matrix Strategy

### Multi-Platform Testing

```yaml
# .github/workflows/test-matrix.yml
name: Test Matrix

on: [push, pull_request]

jobs:
  test:
    strategy:
      matrix:
        # Operating systems
        os: [ubuntu-latest, macos-latest, windows-latest]

        # Rust versions (MSRV + latest)
        rust: ['1.70.0', 'stable', 'nightly']

        # Features combinations
        features:
          - 'default'
          - 'logging,metrics'
          - 'compression,caching'
          - 'all'

        # Python versions (for binding tests)
        python: ['3.8', '3.9', '3.10', '3.11', '3.12']

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      - uses: actions/setup-python@v4
        with:
          python-version: ${{ matrix.python }}

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Run core tests
        run: |
          cargo test --all-features \
            --features ${{ matrix.features }}

      - name: Run core coverage
        if: matrix.rust == 'stable' && matrix.os == 'ubuntu-latest'
        run: |
          cargo tarpaulin --out Xml \
            --fail-under 95

      - name: Upload coverage
        if: matrix.rust == 'stable' && matrix.os == 'ubuntu-latest'
        uses: codecov/codecov-action@v3

      - name: Run binding tests
        if: matrix.python && matrix.os == 'ubuntu-latest'
        run: |
          cd bindings/python
          pytest --cov=mylib --cov-fail-under=80
```

### Continuous Benchmarking

```yaml
# .github/workflows/benchmarks.yml
name: Benchmarks

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable

      - name: Run benchmarks
        run: cargo bench --all-features

      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/output.txt
          github-token: ${{ secrets.GITHUB_TOKEN }}
          auto-push: true
```

## Testing Best Practices

### Error Testing

```rust
#[test]
fn test_error_display() {
    let err = Error::Timeout("test".to_string());
    assert_eq!(err.to_string(), "Timeout: test");
}

#[test]
fn test_error_conversion() {
    let io_error = std::io::Error::last_os_error();
    let err: Error = io_error.into();
    matches!(err, Error::NetworkError(_));
}
```

### Concurrent Testing

```rust
#[tokio::test]
async fn test_concurrent_requests() {
    let client = Arc::new(Client::new(Config::default()).unwrap());

    let mut handles = vec![];
    for _ in 0..10 {
        let client = Arc::clone(&client);
        handles.push(tokio::spawn(async move {
            client.execute(request).await
        }));
    }

    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

### Timeout Testing

```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_timeout_enforcement() {
    let config = Config {
        timeout_ms: 100,
        ..Default::default()
    };
    let client = Client::new(config).unwrap();

    // This should timeout
    let result = tokio::time::timeout(
        Duration::from_millis(500),
        client.execute(slow_request())
    ).await;

    assert!(result.is_err());
}
```

## Cross-References

- **Rust Core Design**: See [02-rust-core-design.md](02-rust-core-design.md)
- **Binding Patterns**: See [03-binding-patterns.md](03-binding-patterns.md)
- **Build System**: See [04-build-system.md](04-build-system.md)
- **Performance Patterns**: See [06-performance-patterns.md](06-performance-patterns.md)
- **Security Model**: See [07-security-model.md](07-security-model.md)

## Implementation Checklist

- [ ] 95% coverage achieved in Rust core
- [ ] 80% coverage achieved in all bindings
- [ ] API parity fixtures defined and loading
- [ ] All 10 bindings implement fixture tests
- [ ] CI test matrix covers MSRV + latest
- [ ] Coverage reports uploaded to codecov
- [ ] Benchmark regression detection enabled
- [ ] Error paths fully tested
- [ ] Concurrent access patterns tested
- [ ] Timeout enforcement validated
