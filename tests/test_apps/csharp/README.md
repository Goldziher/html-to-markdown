# C# Test App for html-to-markdown

Tests the published html-to-markdown package from NuGet.

## Setup

```bash
dotnet restore
```

## Run Tests

```bash
# Smoke tests
dotnet test --filter FullyQualifiedName~SmokeTest

# Comprehensive tests
dotnet test --filter FullyQualifiedName~ComprehensiveTest

# All tests
dotnet test
```
