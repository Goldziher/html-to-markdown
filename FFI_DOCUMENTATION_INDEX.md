# FFI Documentation Index

## Overview
This collection of documents provides comprehensive analysis of the html-to-markdown FFI architecture and a detailed implementation guide for building the visitor FFI bridge.

## Document Map

### 1. **FFI_ARCHITECTURE_REPORT.md** (Primary Reference)
**Status**: Complete architectural analysis
**Length**: ~700 lines
**Audience**: Architects, senior developers, FFI implementers

**Contents**:
- Current FFI module organization & exports
- Thread-local LAST_ERROR pattern & error flow
- String marshalling utilities (bytes_to_c_string, string_to_c_string)
- Current conversion functions (3 API tiers)
- Profiling & panic handling mechanisms
- Available dependencies & transitive deps
- Integration points for visitor module
- Type marshalling challenges for visitor
- Callback registration pattern recommendations
- cbindgen configuration details
- Key architectural patterns to follow

**Key Sections**:
- Section 2: Error Handling Pattern (thread-safety, panic-wrapping)
- Section 3: String Marshalling (UTF-8 validation, ownership transfer)
- Section 4: Conversion Functions (Simple, with-length, bytes variants)
- Section 7: Integration Points (where to add visitor module)
- Section 10: Header Generation (cbindgen config)
- Section 9: Key Architectural Patterns (null checks, error clearing, panic wrapping)

**When to Use**: Start here for understanding current architecture

---

### 2. **FFI_QUICK_REFERENCE.md** (Implementation Reference)
**Status**: Code patterns & templates
**Length**: ~500 lines
**Audience**: Developers implementing FFI functions

**Contents**:
- Error handling template (step-by-step)
- String marshalling patterns (4 variants)
- Opaque handle pattern (for visitor)
- Byte buffer API pattern
- Error checking in C
- Null pointer checks
- UTF-8 validation patterns
- Memory leak prevention
- Feature gating examples
- cbindgen configuration updates
- Common mistakes & how to avoid them

**Key Sections**:
- "Error Handling Template" - Use this for every FFI function
- "String Marshalling Patterns" - Shows all 4 common variants
- "Opaque Handle Pattern" - For visitor implementation
- "Common Mistakes to Avoid" - Prevent bugs before they happen

**When to Use**: Reference while writing FFI code

---

### 3. **VISITOR_FFI_IMPLEMENTATION_GUIDE.md** (Step-by-Step Guide)
**Status**: Detailed implementation roadmap
**Length**: ~600 lines
**Audience**: Developers implementing visitor FFI

**Contents**:
- Phase 1: Type Definitions (VisitorCallbacks, VisitorHandle, callback types)
- Phase 2: Visitor Wrapper (CVisitorAdapter, callback marshalling)
- Phase 3: Integration (convert_with_visitor function)
- Phase 4: Module Integration (lib.rs updates, Cargo.toml)
- Phase 5: Testing (unit tests, integration tests)
- Phase 6: Documentation (C header comments, example programs)
- Complete checklist for implementation
- Next steps & references

**Key Sections**:
- Phase 1: Data structure design with complete code
- Phase 2: Full CVisitorAdapter implementation template
- Phase 5: Unit test patterns with callback tracking
- Checklist: All tasks needed to complete implementation

**When to Use**: During implementation, follow each phase sequentially

---

### 4. **VISITOR_FFI_DEPENDENCIES.md** (Technical Decisions)
**Status**: Analysis of dependencies & architecture decisions
**Length**: ~400 lines
**Audience**: Architects, tech leads

**Contents**:
- Current FFI crate dependencies (all of them)
- Why NO new dependencies needed
- 10 key architectural decisions with rationale & trade-offs
- Dependency chain diagram
- Core library feature support status
- Integration points with existing code
- Build impact (time, binary size, platforms)
- Testing dependencies
- Performance characteristics
- Version compatibility
- Implementation priority (high/medium/low)

**Key Sections**:
- "NO New Dependencies Required" - Good news summary
- "Key Architectural Decisions" (all 10 with trade-offs)
- "Integration Points with Existing Code" - How to reuse existing modules
- "Performance Characteristics" - Callback overhead analysis
- "Implementation Priority" - What to build first

**When to Use**: Review before implementation, reference for decisions

---

## Quick Navigation by Task

### "I want to understand the current FFI architecture"
→ Start with **FFI_ARCHITECTURE_REPORT.md**, Section 1-5

### "I'm writing a new FFI function"
→ Use **FFI_QUICK_REFERENCE.md**, "Error Handling Template" section
→ Reference **FFI_ARCHITECTURE_REPORT.md**, Section 9 for patterns

### "I'm implementing the visitor FFI"
→ Follow **VISITOR_FFI_IMPLEMENTATION_GUIDE.md** phase by phase
→ Use **FFI_QUICK_REFERENCE.md** for code patterns
→ Reference **VISITOR_FFI_DEPENDENCIES.md** for decisions

### "I need to understand error handling"
→ **FFI_ARCHITECTURE_REPORT.md**, Section 2
→ **FFI_QUICK_REFERENCE.md**, "Error Handling Template"
→ Example in **VISITOR_FFI_IMPLEMENTATION_GUIDE.md**, Phase 2

### "I'm integrating with language bindings"
→ **FFI_ARCHITECTURE_REPORT.md**, Section 7 (integration points)
→ **FFI_QUICK_REFERENCE.md** (C usage patterns)
→ Generated C header: `/crates/html-to-markdown-ffi/html_to_markdown.h`

### "I need to understand the type marshalling"
→ **FFI_ARCHITECTURE_REPORT.md**, Section 3 (string utilities)
→ **FFI_ARCHITECTURE_REPORT.md**, Section 7.4 (type mapping challenges)
→ **FFI_QUICK_REFERENCE.md**, "String Marshalling Patterns"

### "I want to understand design decisions"
→ **VISITOR_FFI_DEPENDENCIES.md**, "Key Architectural Decisions"
→ Each decision includes rationale & trade-offs

---

## Document Structure Overview

```
FFI Documentation
├── FFI_ARCHITECTURE_REPORT.md (MAIN - 12 sections)
│   ├── Current Architecture (Sections 1-6)
│   ├── Patterns & Integration (Sections 7-9)
│   ├── Summary & References (Section 10-12)
│
├── FFI_QUICK_REFERENCE.md (PATTERNS - for development)
│   ├── Templates (error handling, strings, handles)
│   ├── Patterns (byte buffers, feature gating)
│   ├── Mistakes (what to avoid)
│
├── VISITOR_FFI_IMPLEMENTATION_GUIDE.md (ROADMAP - phases 1-6)
│   ├── Design Phase 1 (types)
│   ├── Implementation Phases 2-4 (code)
│   ├── Testing Phase 5 (tests)
│   ├── Documentation Phase 6 (docs)
│
├── VISITOR_FFI_DEPENDENCIES.md (DECISIONS - why & how)
│   ├── Dependencies analysis
│   ├── Architectural decisions (10 total)
│   ├── Integration & impact analysis
│
└── FFI_DOCUMENTATION_INDEX.md (THIS FILE)
    ├── Document map
    ├── Navigation guide
    ├── Cross-references
```

---

## Key Files Referenced

**FFI Implementation**:
- `/crates/html-to-markdown-ffi/src/lib.rs` - Main exports
- `/crates/html-to-markdown-ffi/src/error.rs` - Error handling
- `/crates/html-to-markdown-ffi/src/strings.rs` - String utilities
- `/crates/html-to-markdown-ffi/src/conversion.rs` - Basic conversions
- `/crates/html-to-markdown-ffi/src/metadata.rs` - Metadata support
- `/crates/html-to-markdown-ffi/cbindgen.toml` - C header config
- `/crates/html-to-markdown-ffi/Cargo.toml` - Dependencies

**Core Library**:
- `/crates/html-to-markdown/src/visitor.rs` - Visitor trait (60+ methods)
- `/crates/html-to-markdown/src/lib.rs` - Module structure

**Configuration**:
- `/Cargo.toml` - Workspace settings
- `/CLAUDE.md` - Project rules & guidelines

**Generated**:
- `/crates/html-to-markdown-ffi/html_to_markdown.h` - C API header

---

## Recommended Reading Order

### For New Team Members
1. **VISITOR_FFI_DEPENDENCIES.md** (15 min) - Understand decisions
2. **FFI_ARCHITECTURE_REPORT.md** Sections 1-6 (30 min) - Current state
3. **FFI_QUICK_REFERENCE.md** skim (10 min) - Know where patterns are

### For FFI Implementation
1. **FFI_ARCHITECTURE_REPORT.md** Section 9 (15 min) - Key patterns
2. **VISITOR_FFI_DEPENDENCIES.md** sections 1-3 (20 min) - Decisions
3. **VISITOR_FFI_IMPLEMENTATION_GUIDE.md** (read full, 40 min)
4. **FFI_QUICK_REFERENCE.md** (use while coding)

### For Language Binding Authors
1. **FFI_ARCHITECTURE_REPORT.md** Sections 1-4, 7-10 (40 min)
2. **Generated header** (`html_to_markdown.h`) (10 min)
3. **FFI_QUICK_REFERENCE.md** sections on C usage (15 min)

### For Performance Optimization
1. **VISITOR_FFI_DEPENDENCIES.md** "Performance Characteristics" (10 min)
2. **FFI_ARCHITECTURE_REPORT.md** Section 5 (Profiling) (10 min)
3. **FFI_QUICK_REFERENCE.md** "Common Mistakes" (10 min)

---

## Document Maintenance

### When to Update These Docs
- [ ] New FFI functions added (update Architecture Report, Quick Reference)
- [ ] Error handling changes (update Architecture Report Section 2)
- [ ] New dependencies added (update Dependencies doc)
- [ ] cbindgen config changes (update Architecture Report Section 10)
- [ ] Visitor implementation complete (mark Implementation Guide complete)
- [ ] Performance characteristics change (update Dependencies)

### Version Control
- All documentation is committed to git
- Changes go through PR review (must match project standards)
- Update date in document headers if modified
- Keep cross-references consistent

---

## Related Documentation

**In CLAUDE.md**:
- FFI binding pattern (Language Binding Pattern section)
- C FFI library specification
- Code quality standards (Prek hooks)
- Testing requirements (dual testing strategy)

**In Project README**:
- FFI binary download links
- Language-specific binding documentation
- Examples and tutorials

**Generated Documentation**:
- C header: `crates/html-to-markdown-ffi/html_to_markdown.h`
- Rust docs: `cargo doc --open -p html-to-markdown-ffi`

---

## Common Questions & Answers

**Q: Where do I find error handling patterns?**
A: FFI_ARCHITECTURE_REPORT.md Section 2, or FFI_QUICK_REFERENCE.md "Error Handling Template"

**Q: How do I implement a new visitor callback?**
A: Follow VISITOR_FFI_IMPLEMENTATION_GUIDE.md Phase 2, use FFI_QUICK_REFERENCE.md

**Q: What dependencies does visitor FFI add?**
A: None! See VISITOR_FFI_DEPENDENCIES.md "NO New Dependencies Required"

**Q: Why did we choose opaque handles?**
A: See VISITOR_FFI_DEPENDENCIES.md "Key Architectural Decisions" Decision #3

**Q: How do I ensure memory safety?**
A: FFI_ARCHITECTURE_REPORT.md Section 9 (Key Patterns), especially null checks & error clearing

**Q: What about thread safety?**
A: FFI_ARCHITECTURE_REPORT.md Section 2 (LAST_ERROR is thread-local)

**Q: How do I test FFI code?**
A: VISITOR_FFI_IMPLEMENTATION_GUIDE.md Phase 5, or FFI_QUICK_REFERENCE.md test patterns

**Q: What's the performance impact?**
A: VISITOR_FFI_DEPENDENCIES.md "Performance Characteristics" section

---

## Document Statistics

| Document | Lines | Sections | Code Blocks | Focus |
|----------|-------|----------|-------------|-------|
| FFI_ARCHITECTURE_REPORT.md | ~900 | 12 | 50+ | Analysis & Reference |
| FFI_QUICK_REFERENCE.md | ~500 | 13 | 60+ | Patterns & Templates |
| VISITOR_FFI_IMPLEMENTATION_GUIDE.md | ~600 | 6 phases | 40+ | Implementation Roadmap |
| VISITOR_FFI_DEPENDENCIES.md | ~400 | 13 | 15+ | Decisions & Impact |

**Total**: ~2,400 lines of documentation with 160+ code examples

---

## Getting Help

**For Architecture Questions**: FFI_ARCHITECTURE_REPORT.md (Section numbers in each decision)

**For Implementation Help**: VISITOR_FFI_IMPLEMENTATION_GUIDE.md (follow phases)

**For Code Examples**: FFI_QUICK_REFERENCE.md (search for pattern name)

**For Design Rationale**: VISITOR_FFI_DEPENDENCIES.md (Key Architectural Decisions)

**For Generated C API**: `/crates/html-to-markdown-ffi/html_to_markdown.h`

---

**Last Updated**: 2025-12-26
**Applies To**: html-to-markdown v2.17.0+
**Rust Version**: 1.85+ (2024 edition)
