______________________________________________________________________

## priority: critical

# Go 1.25+ Standards

**Go 1.25+ · Table-driven tests · golangci-lint · Error wrapping · Black-box testing**

- Go 1.25+; error wrapping with fmt.Errorf("%w", err), errors.Is/As for checking
- Testing: \*\_test.go with \_test package suffix (black-box), table-driven with t.Run()
- golangci-lint: errcheck, govet, staticcheck, gosec, gocyclo (complexity ≤25)
- Coverage 80%+ on business logic; go test -race for concurrency bugs
- Package structure: cmd/, internal/ (no cross-service imports), pkg/ for libraries
- Naming: PascalCase (types), camelCase (vars), SCREAMING_SNAKE_CASE (consts)
- Context.Context first param in I/O funcs; respect cancellation with select \<-ctx.Done()
- Structured logging: zerolog with Str/Int/Err chaining
- Never: bare error returns, cross-service internal imports, panic in libraries
