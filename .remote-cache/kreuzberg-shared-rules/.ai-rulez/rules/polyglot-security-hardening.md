______________________________________________________________________

## priority: critical

# Polyglot Security Hardening

**CVE disclosure · Multi-language audits · Unsafe code review · Fuzzing · OWASP compliance**

## Core Principle

Security is a shared responsibility across all language ecosystems. Rust core, Python bindings, JavaScript/TypeScript bindings, and all language extensions must maintain critical security standards with equivalent rigor across all implementations.

## CVE Disclosure Policy

### Private Disclosure Period

- **Duration**: 90 days minimum from vulnerability report receipt
- **Reporting channel**: security@kreuzberg.dev (or project security contact)
- **Scope**: All reported CVEs before public disclosure
- **Exceptions**: Zero-day exploits in active exploitation may require faster response

### Disclosure Timeline

1. **Day 0**: Vulnerability report received
1. **Day 1**: Acknowledge receipt, assign security team member
1. **Day 7**: Root cause analysis complete, fix in progress
1. **Day 30**: Fix implemented in main branch (not released)
1. **Day 89**: Release with fix prepared and tested
1. **Day 90**: Public disclosure + simultaneous release across all ecosystems
1. **Day 91**: CVE advisory published, security bulletin issued

### Public Disclosure Components

- **GitHub Security Advisory**: Detailing vulnerability and fix
- **Changelog entry**: All affected versions and mitigation steps
- **Release notes**: Security-focused message in primary language ecosystem
- **Email notification**: Package registry maintainers and downstream projects
- **Blog post**: Technical details (after 90-day embargo)

## Multi-Language Security Audit Pipeline

### Rust Ecosystem Auditing

**cargo-audit checks in CI**:

```yaml
- name: Run cargo-audit
  run: cargo audit --deny warnings
```

Requirements:

- All dependencies must pass audit
- No published vulnerabilities allowed
- RUSTSEC advisories checked on every commit
- SAFETY comments required for unsafe code (see Unsafe Code Review)

### Python Ecosystem Auditing

**pip-audit in CI**:

```yaml
- name: Run pip-audit
  run: |
    pip install pip-audit
    pip-audit --require-hashes --desc
```

Requirements:

- All Python dependencies audited
- PyPI CVE advisories checked
- No known vulnerabilities in published versions
- Fuzzing targets for FFI boundaries

### JavaScript/TypeScript Ecosystem Auditing

**npm audit in CI**:

```yaml
- name: Run npm audit
  run: |
    npm audit --audit-level=moderate
    npm ci --audit  # Prevent installation of vulnerable deps
```

Requirements:

- npm ecosystem scanned on every commit
- Critical vulnerabilities block build
- Moderate/High vulnerabilities generate reports
- Transitive dependency vulnerabilities tracked

### Go Ecosystem Auditing

**go list with vulnerability checking**:

```yaml
- name: Check Go Vulnerabilities
  run: |
    go install golang.org/x/vuln/cmd/govulncheck@latest
    govulncheck ./...
```

### Ruby Ecosystem Auditing

**bundler-audit in CI**:

```yaml
- name: Run bundler-audit
  run: |
    gem install bundler-audit
    bundle-audit check --update
```

### PHP Ecosystem Auditing

**composer audit in CI**:

```yaml
- name: Run composer audit
  run: composer audit --format=json
```

### Java/Maven Auditing

**OWASP Dependency-Check in CI**:

```yaml
- name: OWASP Dependency-Check
  run: |
    mvn org.owasp:dependency-check-maven:check
```

### .NET/C# Auditing

**NuGet vulnerability scanning**:

```yaml
- name: Check NuGet Vulnerabilities
  run: dotnet list package --vulnerable
```

## OWASP Top 10 Compliance

### Web Bindings Security (JavaScript/TypeScript/Python for Web)

All web-facing bindings must comply with OWASP Top 10 2023:

1. **Broken Access Control**

   - No hardcoded credentials in bindings
   - Input validation at binding boundary
   - Authorization checks enforced

1. **Cryptographic Failures**

   - Use validated crypto libraries only
   - No custom crypto implementations
   - TLS 1.3+ for network communication

1. **Injection**

   - Parameterized inputs for all queries
   - Input sanitization at FFI boundary
   - No shell execution from user input

1. **Insecure Design**

   - Threat model documented for bindings
   - Security review before public API exposure
   - Principle of least privilege in permissions

1. **Security Misconfiguration**

   - Default secure configurations
   - No debug features in production builds
   - Security headers configured

1. **Vulnerable Components**

   - All dependencies audited (see audit pipeline above)
   - Update cadence: security patches within 2 weeks
   - Known vulnerability tracking

1. **Authentication Failures**

   - No authentication in core library (delegated to user)
   - Secure token handling in bindings
   - Multi-factor support documentation

1. **Software & Data Integrity Failures**

   - Signed releases (GPG/cosign)
   - Hash verification for artifacts
   - Checksum validation in installers

1. **Logging & Monitoring Failures**

   - Security events logged (not audit trail, unless explicitly enabled)
   - Error messages don't leak sensitive data
   - Monitoring integration documented

1. **SSRF/XXE/External Integrations**

   - No arbitrary URL fetching from bindings
   - XML parsing restricted to safe parsers
   - External service integrations documented

### Compliance Verification

- **Security audit**: Annual third-party security review
- **Checklist enforcement**: Pre-release security validation
- **Penetration testing**: Annual pen test for web bindings
- **Code review**: Security-trained reviewers for all changes

## Unsafe Code Review Requirements

### Mandatory SAFETY Comments

Every `unsafe` block in Rust must have a SAFETY comment:

```rust
// SAFETY: This is safe because:
// 1. We validated that ptr is non-null and properly aligned
// 2. The memory at ptr is initialized with valid T
// 3. No other references exist to this memory (exclusive access)
// 4. T::drop is safe to call on the pointed value
unsafe { ptr::drop_in_place(ptr) }
```

### SAFETY Comment Guidelines

Required elements:

1. **Why it's safe**: Explicit justification
1. **Preconditions met**: What assumptions we make
1. **No data races**: Ensure thread-safety guarantees
1. **No UB triggers**: Verify no undefined behavior
1. **Lifetime validity**: Confirm all references valid

### Unsafe Code Audit

- All unsafe blocks flagged in code review
- SAFETY comments mandatory (CI check)
- Alternative safe implementations required (with justification for unsafe choice)
- Security audit includes unsafe code review
- Unsafe code percentage tracked (goal: < 5% of core)

### Unsafe Code Policy

- Minimize unsafe code: Prefer safe alternatives
- Document assumptions: Explicit preconditions
- Test thoroughly: Unit tests + integration tests for unsafe code
- Fuzz inputs: Fuzzing harnesses for unsafe FFI boundaries
- Review regularly: Audit unsafe code quarterly

## Fuzzing Requirements

### cargo-fuzz Targets

Required fuzzing targets in `fuzz/fuzz_targets/`:

```rust
// fuzz/fuzz_targets/fuzz_parse.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use kreuzberg::parser;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parser::parse(s);
    }
});
```

Fuzzing targets must cover:

1. **Parser fuzzing**: Random input to parse functions
1. **FFI boundary fuzzing**: C ABI fuzzing for bindings
1. **HTML edge cases**: Malformed, nested, extreme size inputs
1. **Unicode handling**: Invalid UTF-8, surrogate pairs, BOM
1. **Performance limits**: Very large inputs (memory, CPU)

### Fuzzing CI Integration

```yaml
- name: Run Cargo Fuzz
  run: |
    cargo install cargo-fuzz
    cargo +nightly fuzz run fuzz_parse -- -max_len=10000 -timeout=5
    cargo +nightly fuzz run fuzz_bindings -- -max_len=10000 -timeout=5
```

- Fuzzing runs on: every commit to main, weekly extended runs
- Coverage metrics: Track fuzzing code coverage
- Regression tests: New fuzzing findings added as test cases
- Reproduction cases: Preserve all fuzzing crashes

### Property-Based Testing

Use proptest for property-based security testing:

```rust
#[cfg(test)]
mod prop_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn parse_never_crashes(s in "\\PC*") {
            let _ = kreuzberg::parse(&s);
        }
    }
}
```

## Security Testing Checklist

### Pre-Release Security Validation

Before every release, verify:

- [ ] All cargo-audit checks pass (zero warnings)
- [ ] pip-audit scans clean for Python packages
- [ ] npm audit shows no critical vulnerabilities
- [ ] govulncheck passes for Go code
- [ ] bundler-audit clean for Ruby gems
- [ ] composer audit passes for PHP packages
- [ ] OWASP dependency-check clean
- [ ] NuGet vulnerability scan clean
- [ ] All unsafe code has SAFETY comments
- [ ] Unsafe code percentage < 5%
- [ ] Fuzzing harnesses run without crash
- [ ] New fuzzing corpus files committed
- [ ] OWASP Top 10 compliance verified
- [ ] Security review completed (internal or third-party)
- [ ] No hardcoded secrets in code or bindings
- [ ] Error messages don't leak sensitive data
- [ ] Cryptographic functions validated
- [ ] Input validation at FFI boundaries
- [ ] No known CVEs in dependencies
- [ ] Security advisory policy document updated
- [ ] Changelog includes security fixes

### Post-Release Monitoring

After release:

- [ ] GitHub security advisory published (if CVE)
- [ ] Downstream projects notified
- [ ] Security mailing list updated
- [ ] Package registries updated with security info
- [ ] Monitoring enabled for new vulnerabilities
- [ ] Security fixes backported to LTS versions

## Agent Coordination

The **security-auditing-specialist** agent is responsible for:

- Running security audit tools before releases
- Reviewing SAFETY comments in unsafe code
- Validating OWASP compliance
- Coordinating security vulnerability disclosures
- Maintaining fuzzing infrastructure
- Conducting regular security reviews

## Never

- Release with known CVEs in dependencies
- Add unsafe code without SAFETY comments
- Skip cargo-audit, pip-audit, npm audit runs
- Use custom cryptographic implementations
- Hardcode secrets or credentials in bindings
- Return unvalidated error messages to external callers
- Skip fuzzing targets for FFI boundaries
- Release before security checklist passes
- Ignore SAFETY comment CI requirements
