# Go Test App for html-to-markdown

Tests the published html-to-markdown package from pkg.go.dev.

## Setup

```bash
go mod download
```

## Run Tests

```bash
# Smoke tests
go test -v -run Smoke

# Comprehensive tests
go test -v -run Comprehensive

# All tests
go test -v
```
