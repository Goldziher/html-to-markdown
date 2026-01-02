# Customizing AI-Rulez for Your Project

AI-Rulez is designed to be extended and customized for project-specific needs. This guide explains how consuming projects can create custom agents, rules, skills, and profiles.

## Extension Architecture

The extension system operates in three layers:

1. **Core (ai-rulez):** Provides domains, agents, skills, and rules for polyglot Rust development
1. **Extension Points (config.yaml):** Defines where projects can inject custom behavior
1. **Custom Configuration (.ai-rulez/):** Project-specific extensions that augment core functionality

## Directory Structure

Your consuming project should have a `.ai-rulez/` directory at the root:

```
your-project/
├── .ai-rulez/
│   ├── config.yaml              # Project-specific configuration
│   ├── custom-agents.yaml       # Custom AI agent definitions
│   ├── custom-rules.yaml        # Project-specific development rules
│   ├── custom-profiles.yaml     # Custom profile combinations
│   ├── custom-skills/           # Custom skill implementations
│   │   ├── my-skill.yaml
│   │   ├── my-skill.md
│   │   └── ...
│   └── domains/                 # Optional: custom domain definitions
├── Cargo.toml
├── pyproject.toml
└── ...
```

## 1. Custom Agents (custom-agents.yaml)

Define project-specific AI agents that supplement the core ai-rulez agents.

### Template

```yaml
# .ai-rulez/custom-agents.yaml
agents:
  # Example: Custom domain expert for your project
  domain-expert:
    name: "Domain Expert"
    description: "Understands project-specific architecture and patterns"
    model: "claude-haiku-4-5"
    role: "domain-expert"
    context:
      - file_patterns:
          - "src/**/*.rs"
          - "docs/ARCHITECTURE.md"
      - skill_context: ["rust-core-arch", "your-domain-knowledge"]
    instructions: |
      You are the domain expert for {project_name}.

      Key responsibilities:
      - Understand and maintain project-specific patterns
      - Ensure consistency with domain requirements
      - Guide architectural decisions within the domain
      - Review code for domain-specific concerns

  # Example: Custom security specialist
  security-specialist-custom:
    name: "Security Specialist (Project-Specific)"
    description: "Reviews security practices specific to {project_name}"
    model: "claude-sonnet-4-5"
    role: "security-specialist"
    context:
      - file_patterns:
          - "src/**/*.rs"
          - "bindings/**/*.py"
          - "security/**.md"
      - skill_context: ["security-and-vulnerability-management", "your-security-rules"]
    instructions: |
      You are the security specialist for {project_name}.

      Focus on:
      - FFI boundary security (for binding projects)
      - Dependency vulnerability scanning
      - Cryptographic usage patterns
      - Data sensitivity classification
```

### Using Custom Agents

After defining custom agents, reference them in:

1. **Custom profiles** (see Custom Profiles section)
1. **Task assignments** in your AI workflow
1. **Rules that trigger specific agents**

______________________________________________________________________

## 2. Custom Rules (custom-rules.yaml)

Define project-specific development rules that guide AI behavior.

### Template

```yaml
# .ai-rulez/custom-rules.yaml
rules:
  # Example: Project-specific naming convention
  project-module-naming-standard:
    name: "Project Module Naming Standard"
    description: "Enforce consistent naming for project-specific modules"
    applies_to:
      - language: "rust"
        scope: "modules"
    guidelines: |
      Module names must follow the pattern: {feature}_{concern}

      Examples:
      - extraction_pdf
      - extraction_ocr
      - transform_pipeline

      Avoid:
      - Generic names like "utils" or "helpers"
      - Abbreviations (unless widely understood)
      - Plural forms for modules
    validation:
      - pattern: "^[a-z]+(_[a-z]+)*$"
      - length_max: 32
    agents:
      - rust-core-engineer
      - code-reviewer

  # Example: API stability requirement
  api-stability-guarantee:
    name: "API Stability Guarantee"
    description: "Ensure public APIs maintain backward compatibility"
    applies_to:
      - language: "rust"
        scope: "public-apis"
    guidelines: |
      Public API changes are restricted to major version increments.

      Permitted in minor versions:
      - New public methods
      - New public modules
      - Performance optimizations

      Breaking changes require:
      - Deprecation period in prior version
      - Migration guide in release notes
      - Major version bump
    agents:
      - polyglot-architect
      - code-reviewer
      - api-doc-writer

  # Example: Binding-specific testing requirement
  binding-feature-parity-validation:
    name: "Binding Feature Parity Validation"
    description: "Ensure all language bindings implement the same features"
    applies_to:
      - scope: "language-bindings"
    guidelines: |
      Every public API in Rust must be exposed in all active language bindings.

      Implementation checklist:
      1. Rust core API defined and documented
      2. Python binding implemented and tested
      3. TypeScript binding implemented and tested
      4. Integration tests for feature across languages
      5. Documentation parity (examples in each language)

      Exceptions require polyglot-architect approval.
    agents:
      - polyglot-architect
      - code-reviewer
      - test-automation-engineer

  # Example: Documentation requirement
  documentation-completeness:
    name: "Documentation Completeness Standard"
    description: "Ensure all public APIs are documented with examples"
    applies_to:
      - scope: "documentation"
    guidelines: |
      Every public API requires:
      - Module-level documentation (/// comment)
      - Function/struct documentation
      - At least one usage example
      - Links to related APIs
      - Error conditions documented

      For bindings:
      - Python: docstrings with type hints
      - TypeScript: JSDoc comments
      - Ruby: YARD documentation
    agents:
      - api-doc-writer
      - docs-writer
```

### Key Rule Fields

- **name**: Human-readable rule name
- **description**: What this rule governs
- **applies_to**: Language and scope filters
- **guidelines**: The actual rule text (can reference variables like {project_name})
- **validation**: Optional: regex patterns, length limits, etc.
- **agents**: Which agents should enforce this rule

______________________________________________________________________

## 3. Custom Skills (custom-skills/ directory)

Define reusable AI capabilities and knowledge bases for your project.

### Skill File Structure

Each custom skill requires two files:

1. **skill-name.yaml** - Metadata and configuration
1. **SKILL.md** - Detailed documentation

### Template: skill.yaml

```yaml
# .ai-rulez/custom-skills/my-skill.yaml
name: "My Custom Skill"
id: "my-custom-skill"
category: "domain-specific"
version: "1.0.0"

description: |
  Comprehensive guide to {feature} in {project_name}.

  This skill enables AI to understand and implement:
  - Feature architecture and design patterns
  - Common use cases and pitfalls
  - Integration points with other systems

applies_to:
  agents:
    - domain-expert
    - rust-core-engineer
  languages:
    - rust
    - python
    - typescript
  roles:
    - implementation
    - architecture

dependencies:
  - "rust-core-arch"
  - "polyglot-error-handling-standardization"

learning_objectives:
  - "Understand the architecture of {feature}"
  - "Implement {feature} correctly"
  - "Design extensions to {feature}"
  - "Debug common {feature} issues"

related_skills:
  - "error-handling-strategy"
  - "workspace-structure-project-organization"
```

### Template: SKILL.md

```markdown
# My Custom Skill

## Overview

Explain the feature/concept and why it's important to your project.

## Architecture

### System Design

Describe the high-level architecture:

- Component A: Responsibility
- Component B: Responsibility
- Integration point: How components interact

### Data Flow

Document key data flows:

```

Input → Processing → Output
↓ ↓ ↓
Source → Logic → Result

````

## Implementation Patterns

### Pattern 1: Common Use Case

**When to use:** Describe the scenario

**Example:**

```rust
// Rust example
pub fn my_function() {
    // Implementation
}
````

```python
# Python example
def my_function():
    # Implementation
    pass
```

**Key points:**

- Bullet point 1
- Bullet point 2

### Pattern 2: Advanced Use Case

Similar structure for more complex patterns.

## Anti-Patterns

### What NOT to Do

Explain common mistakes and why they're problematic.

## Debugging Guide

### Issue: Common Problem

**Symptoms:** How to identify this issue
**Root cause:** Why it happens
**Solution:** How to fix it

## Performance Considerations

- Point 1
- Point 2

## Related Concepts

- Linked skill 1
- Linked skill 2

## References

- External documentation
- Architecture decision records
- Related issues/PRs

````

### Example: Custom Extraction Skill

```yaml
# .ai-rulez/custom-skills/document-extraction.yaml
name: "Document Extraction Architecture"
id: "document-extraction-arch"
category: "domain-specific"
version: "1.0.0"

description: |
  Comprehensive guide to Kreuzberg's multi-format document extraction system.

  This skill enables AI to understand and implement extraction strategies for:
  - PDF documents (text, images, metadata)
  - Microsoft Office files (Word, Excel, PowerPoint)
  - OCR integration and fallback strategies
  - Plugin-based format extension

applies_to:
  agents:
    - rust-core-engineer
    - python-bindings-engineer
    - code-reviewer
  languages:
    - rust
    - python
  roles:
    - implementation
    - architecture

dependencies:
  - "rust-core-arch"
  - "fixture-driven-testing-strategy"
````

______________________________________________________________________

## 4. Custom Profiles (custom-profiles.yaml)

Combine custom agents, rules, and skills into tailored profiles for specific contexts.

### Template

```yaml
# .ai-rulez/custom-profiles.yaml
profiles:
  # Example: Extraction + Security focus
  extraction-hardened:
    name: "Extraction with Security Focus"
    description: "Profile optimized for security-critical extraction features"

    # Extend an existing profile
    extends: "extraction-library"

    # Override agents
    agents:
      - polyglot-architect
      - rust-core-engineer
      - security-specialist-custom        # Custom agent
      - python-bindings-engineer
      - test-automation-engineer
      - code-reviewer

    # Add custom rules
    priority_rules:
      - rust-2024-edition-core-conversion-engine
      - binding-feature-parity-validation  # Custom rule
      - api-stability-guarantee            # Custom rule
      - polyglot-build-system-distribution
      - security-hardening-extraction      # Custom rule

    # Add custom skills
    skills:
      - "document-extraction-arch"         # Custom skill
      - "security-and-vulnerability-management"

    # Custom model routing
    model_routing:
      architecture_decisions: "claude-sonnet-4-5"
      security_review: "claude-opus-4-5"   # Use stronger model for security
      core_implementation: "claude-haiku-4-5"
      binding_implementation: "claude-haiku-4-5"
      testing: "claude-haiku-4-5"
      code_review: "claude-sonnet-4-5"     # Stronger review for extraction
      documentation: "claude-haiku-4-5"

  # Example: Fast iteration profile
  rapid-iteration:
    name: "Rapid Iteration Profile"
    description: "Optimized for quick iteration cycles with Haiku-only routing"

    extends: "rust-library"

    agents:
      - rust-core-engineer
      - test-automation-engineer
      - code-reviewer

    priority_rules:
      - rust-2024-edition-core-conversion-engine
      - continuous-integration-coverage

    model_routing:
      architecture_decisions: "claude-haiku-4-5"
      core_implementation: "claude-haiku-4-5"
      testing: "claude-haiku-4-5"
      code_review: "claude-haiku-4-5"
      documentation: "claude-haiku-4-5"

  # Example: New contributor onboarding profile
  onboarding:
    name: "New Contributor Onboarding"
    description: "Profile focused on learning and understanding the codebase"

    extends: "conversion-library"

    skills:
      - "quick-start"
      - "workspace-structure-project-organization"
      - "core-principles"
      - "my-custom-skill"                  # Your custom domain skill

    model_routing:
      # Use stronger models for clear explanations
      architecture_decisions: "claude-sonnet-4-5"
      core_implementation: "claude-sonnet-4-5"
      documentation: "claude-sonnet-4-5"
      code_review: "claude-haiku-4-5"
```

______________________________________________________________________

## 5. Extension Points Configuration

Configure which extension points are enabled in your `.ai-rulez/config.yaml`:

```yaml
# .ai-rulez/config.yaml
# (This extends the core config.yaml from ai-rulez)

name: "your-project-name"
type: "project"  # "project" for consuming projects, "shared" for shared modules

# Enable/disable extension points
extension_points:
  custom_agents: true        # Load custom-agents.yaml
  custom_rules: true         # Load custom-rules.yaml
  custom_skills: true        # Load custom-skills/ directory
  custom_profiles: true      # Load custom-profiles.yaml
  custom_domains: false      # Optional: define custom domains

# Reference custom components
custom_agents_file: "custom-agents.yaml"
custom_rules_file: "custom-rules.yaml"
custom_skills_dir: "custom-skills/"
custom_profiles_file: "custom-profiles.yaml"

# Set project-specific default profile
default_profile: "extraction-hardened"

# Add project-specific metadata
project_metadata:
  domain: "document-extraction"
  critical_apis:
    - "pdf-extraction"
    - "ocr-integration"
  security_level: "high"
  language_focus: ["rust", "python"]
```

______________________________________________________________________

## Integration Workflow

### Step 1: Create Base Structure

```bash
mkdir -p .ai-rulez/custom-skills
touch .ai-rulez/config.yaml
touch .ai-rulez/custom-agents.yaml
touch .ai-rulez/custom-rules.yaml
touch .ai-rulez/custom-profiles.yaml
```

### Step 2: Define Custom Agents

Edit `.ai-rulez/custom-agents.yaml`:

```yaml
agents:
  my-domain-expert:
    name: "My Domain Expert"
    description: "Understands project-specific patterns"
    model: "claude-haiku-4-5"
    role: "domain-expert"
    instructions: |
      You are an expert in {project_name}.
```

### Step 3: Define Custom Rules

Edit `.ai-rulez/custom-rules.yaml`:

```yaml
rules:
  my-project-rule:
    name: "My Project Rule"
    description: "Enforces project-specific standards"
    applies_to:
      - language: "rust"
    guidelines: "Project-specific guideline"
    agents:
      - rust-core-engineer
```

### Step 4: Create Custom Skills

Create `.ai-rulez/custom-skills/domain-knowledge.yaml` and `.ai-rulez/custom-skills/SKILL.md`

### Step 5: Combine into Profiles

Edit `.ai-rulez/custom-profiles.yaml`:

```yaml
profiles:
  my-profile:
    extends: "conversion-library"
    agents:
      - my-domain-expert
    priority_rules:
      - my-project-rule
    skills:
      - "domain-knowledge"
```

### Step 6: Enable Configuration

Edit `.ai-rulez/config.yaml`:

```yaml
default_profile: "my-profile"
extension_points:
  custom_agents: true
  custom_rules: true
  custom_skills: true
  custom_profiles: true
```

______________________________________________________________________

## Best Practices

### 1. Naming Conventions

- **Agents:** Use kebab-case descriptive names
  - `domain-expert`, `security-specialist-custom`, `data-processing-engineer`
- **Rules:** Use kebab-case descriptive names
  - `module-naming-standard`, `api-stability-guarantee`
- **Skills:** Use kebab-case with optional suffix
  - `domain-knowledge`, `extraction-architecture`, `custom-optimization-patterns`
- **Profiles:** Use kebab-case descriptive names
  - `extraction-hardened`, `rapid-iteration`, `onboarding`

### 2. Documentation

Every custom agent, rule, and skill must have clear documentation:

- **Agents:** Clear role description and responsibility scope
- **Rules:** Specific guidelines with examples
- **Skills:** Comprehensive guides with patterns and anti-patterns
- **Profiles:** Clear description of when and why to use

### 3. Model Selection

Choose models strategically:

- **Claude Opus 4.5:** Critical decisions (architecture, security, major refactoring)
- **Claude Sonnet 4.5:** Complex tasks (architecture design, code review, learning)
- **Claude Haiku 4.5:** Fast iteration (implementation, testing, documentation)

### 4. Skill Dependencies

Define dependencies on existing ai-rulez skills to maintain consistency:

```yaml
dependencies:
  - "rust-core-arch"                        # Core architecture patterns
  - "polyglot-error-handling-standardization"  # Error handling approaches
  - "testing-philosophy-coverage"            # Testing standards
```

### 5. Scope Management

Keep custom components focused:

- **Agents:** Single primary responsibility
- **Rules:** Address one concern (naming, safety, testing, etc.)
- **Skills:** Comprehensive but focused on one feature/pattern
- **Profiles:** Clear, distinct use case

### 6. Version Control

Commit custom configuration to version control:

```bash
# Include custom configuration
git add .ai-rulez/custom-*.yaml
git add .ai-rulez/custom-skills/
git add .ai-rulez/config.yaml

# Exclude generated/cached files
echo ".claude" >> .gitignore
echo ".ai-cache" >> .gitignore
```

______________________________________________________________________

## Examples

### Example 1: Data Processing Project

```yaml
# .ai-rulez/custom-agents.yaml
agents:
  data-scientist:
    name: "Data Scientist"
    description: "Understands ML/stats requirements"
    model: "claude-sonnet-4-5"
    role: "ml-specialist"
```

```yaml
# .ai-rulez/custom-rules.yaml
rules:
  numerical-stability:
    name: "Numerical Stability Standard"
    description: "Ensure algorithms maintain numerical stability"
    guidelines: |
      All mathematical operations must:
      - Use appropriate precision (f64 for stats)
      - Handle edge cases (NaN, Inf, underflow)
      - Include stability tests
```

```yaml
# .ai-rulez/custom-profiles.yaml
profiles:
  ml-optimized:
    extends: "python-focused"
    agents:
      - data-scientist
    priority_rules:
      - numerical-stability
```

### Example 2: Web Service Project

```yaml
# .ai-rulez/custom-profiles.yaml
profiles:
  api-focused:
    extends: "web-framework"
    agents:
      - polyglot-architect
      - rust-core-engineer
      - python-bindings-engineer
      - typescript-bindings-engineer
    priority_rules:
      - api-stability-guarantee
      - performance-regression-prevention
    model_routing:
      architecture_decisions: "claude-sonnet-4-5"
      core_implementation: "claude-haiku-4-5"
      binding_implementation: "claude-haiku-4-5"
      testing: "claude-haiku-4-5"
      code_review: "claude-sonnet-4-5"  # Stronger review for APIs
      documentation: "claude-haiku-4-5"
```

______________________________________________________________________

## Troubleshooting

### Custom Agents Not Loading

Check:

1. `custom-agents.yaml` exists in `.ai-rulez/`
1. `extension_points.custom_agents: true` in config
1. YAML syntax is valid

### Rules Not Applied

Check:

1. Rule is listed in profile's `priority_rules`
1. `applies_to` conditions match your context
1. Agents specified in rule exist

### Skills Not Available

Check:

1. Skill files exist in `.ai-rulez/custom-skills/`
1. Both `.yaml` and `.md` files present
1. Referenced in a profile's `skills` section

______________________________________________________________________

## See Also

- [PROFILES.md](./PROFILES.md) - Detailed profile documentation
- [config.yaml](./config.yaml) - Core configuration reference
- Extension point definitions in root `config.yaml`
