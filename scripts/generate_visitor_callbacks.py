#!/usr/bin/env python3
"""Generate visitor callback code from YAML schema.

This script generates FFI visitor callback implementations for multiple languages
from a central YAML schema definition. It eliminates ~2,450 lines of duplicated
code across Rust, Go, C#, and Java layers.

Usage:
    python scripts/generate_visitor_callbacks.py

Generated files:
    - crates/html-to-markdown-ffi/src/visitor/registry_generated.rs
    - packages/go/v2/htmltomarkdown/visitor_generated.go
    - packages/csharp/HtmlToMarkdown/Visitor/VisitorBridgeGenerated.cs
    - packages/java/src/main/java/sh/kreuzberg/htmltomarkdown/visitor/VisitorBridgeGenerated.java

Requirements:
    - PyYAML (pip install pyyaml)
    - Jinja2 (pip install jinja2)
"""

import sys
from pathlib import Path
from typing import Any

try:
    import yaml
    from jinja2 import Environment, FileSystemLoader, select_autoescape
except ImportError as e:
    print(f"Error: Missing required dependency: {e}", file=sys.stderr)
    print("Install dependencies: pip install pyyaml jinja2", file=sys.stderr)
    sys.exit(1)


def load_schema(schema_path: Path) -> dict[str, Any]:
    """Load and validate the YAML schema."""
    if not schema_path.exists():
        raise FileNotFoundError(f"Schema file not found: {schema_path}")

    with schema_path.open(encoding="utf-8") as f:
        schema = yaml.safe_load(f)

    # Validate required keys
    required_keys = ["version", "metadata", "types", "callbacks", "generation"]
    for key in required_keys:
        if key not in schema:
            raise ValueError(f"Missing required key in schema: {key}")

    return schema


def setup_jinja_env(template_dir: Path) -> Environment:
    """Set up Jinja2 environment with template directory."""
    if not template_dir.exists():
        raise FileNotFoundError(f"Template directory not found: {template_dir}")

    return Environment(
        loader=FileSystemLoader(str(template_dir)),
        autoescape=select_autoescape(disabled_extensions=("j2",)),
        trim_blocks=True,
        lstrip_blocks=True,
    )


def generate_code(
    env: Environment,
    template_name: str,
    output_path: Path,
    context: dict[str, Any],
) -> None:
    """Generate code from template and write to output file."""
    template = env.get_template(template_name)
    rendered = template.render(**context)

    # Ensure output directory exists
    output_path.parent.mkdir(parents=True, exist_ok=True)

    # Write generated code
    with output_path.open("w", encoding="utf-8") as f:
        f.write(rendered)

    print(f"✓ Generated: {output_path}")


def main() -> None:
    """Generate visitor callback code for all target languages."""
    # Locate project root (go up from scripts/ to project root)
    script_dir = Path(__file__).parent.resolve()
    project_root = script_dir.parent

    # Define paths
    schema_path = project_root / "crates/html-to-markdown-ffi/visitor_callbacks.yaml"
    template_dir = project_root / "crates/html-to-markdown-ffi/templates"

    print("HTML-to-Markdown Visitor Callback Code Generator")
    print("=" * 60)
    print(f"Schema: {schema_path.relative_to(project_root)}")
    print(f"Templates: {template_dir.relative_to(project_root)}")
    print()

    # Load schema
    try:
        schema = load_schema(schema_path)
    except Exception as e:
        print(f"Error loading schema: {e}", file=sys.stderr)
        sys.exit(1)

    # Set up Jinja2 environment
    try:
        env = setup_jinja_env(template_dir)
    except Exception as e:
        print(f"Error setting up templates: {e}", file=sys.stderr)
        sys.exit(1)

    # Prepare template context
    context = {
        "version": schema["version"],
        "metadata": schema["metadata"],
        "types": schema["types"],
        "callbacks": schema["callbacks"],
    }

    # Generate code for each target language
    targets = schema["generation"]["targets"]
    total_callbacks = schema["metadata"]["total_callbacks"]

    print(f"Generating code for {total_callbacks} callbacks across {len(targets)} languages:")
    print()

    generated_files = []
    for lang_name, lang_config in targets.items():
        template_file = Path(lang_config["template"]).name
        output_file = project_root / lang_config["output_file"]

        try:
            generate_code(env, template_file, output_file, context)
            generated_files.append(output_file)
        except Exception as e:
            print(f"✗ Error generating {lang_name}: {e}", file=sys.stderr)
            sys.exit(1)

    # Summary
    print()
    print("=" * 60)
    print(f"✓ Successfully generated {len(generated_files)} files")
    print()
    print("Expected reductions:")
    for reduction in schema["metadata"]["expected_reduction"].values():
        print(f"  • {reduction}")
    print()
    print("Next steps:")
    print("  1. Review generated files")
    print("  2. Run: task rust:build")
    print("  3. Run: task test")
    print("  4. Commit changes")


if __name__ == "__main__":
    main()
