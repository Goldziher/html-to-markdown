# Migration Guide: TypeScript v2.18.x → v2.19.0

## Breaking Change: Scoped npm Packages

In v2.19.0, npm packages were moved to the `@kreuzberg` scope to align with the Kreuzberg.dev organization.

### Package Installation Update

**Before (v2.18.x):**
```bash
npm install html-to-markdown-node
npm install html-to-markdown-wasm
```

**After (v2.19.0+):**
```bash
npm install @kreuzberg/html-to-markdown-node
npm install @kreuzberg/html-to-markdown-wasm
```

### Import Statement Update

**Before:**
```typescript
import { convert } from 'html-to-markdown-node';
import { convert } from 'html-to-markdown-wasm';
```

**After:**
```typescript
import { convert } from '@kreuzberg/html-to-markdown-node';
import { convert } from '@kreuzberg/html-to-markdown-wasm';
```

### TypeScript Declaration Update

Update your TypeScript configuration if you have imports from the old package name:

**Before (tsconfig.json or import aliases):**
```json
{
  "compilerOptions": {
    "paths": {
      "html-to-markdown": ["node_modules/html-to-markdown-node"]
    }
  }
}
```

**After:**
```json
{
  "compilerOptions": {
    "paths": {
      "@kreuzberg/html-to-markdown": ["node_modules/@kreuzberg/html-to-markdown-node"]
    }
  }
}
```

### Deno Update

**Before:**
```typescript
import { convert } from "npm:html-to-markdown-wasm";
```

**After:**
```typescript
import { convert } from "npm:@kreuzberg/html-to-markdown-wasm";
```

## Summary of Changes

- All npm packages now use `@kreuzberg` scope
- `html-to-markdown-node` → `@kreuzberg/html-to-markdown-node`
- `html-to-markdown-wasm` → `@kreuzberg/html-to-markdown-wasm`
- TypeScript types and APIs are identical
- No functional changes to the library
