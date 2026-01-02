______________________________________________________________________

## priority: critical

# TypeScript Strictest Standards

**TypeScript 5.x · Strictest typing · No any/object · Generics required · Tests next to source**

- Enable ALL strict flags: strict, noUncheckedIndexedAccess, exactOptionalPropertyTypes
- Ban any and object types; use unknown with guards, Record\<string, unknown>
- Generics with constraints: <T extends BaseType>, satisfies operator, const assertions
- Tests: .spec.ts next to source files (NOT __tests__/); vitest, 80%+ coverage
- Functional: pure functions over classes, map/filter/reduce, immutability, readonly
- Nullish coalescing ??, optional chaining ?., type predicates (x is Type)
- Import type for types, organize by feature, path aliases (@/lib/\*)
- Biome for linting/formatting, pnpm ≥10.17, pnpm-lock.yaml committed
- React: function components, custom hooks (use\*), proper prop typing
- Never: any/object types, __test__ dirs, non-null assertions !, || for defaults
