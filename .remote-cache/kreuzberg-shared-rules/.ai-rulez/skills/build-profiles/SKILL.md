______________________________________________________________________

## priority: critical

# Build Profiles

**BUILD_PROFILE environment variable controls build mode** (default: "release").

**Available Profiles**:

1. **dev**: Fast iteration, debug symbols, no optimizations. Use for development/testing.

   - Cargo profile: debug
   - Optimization: -C opt-level=0
   - Usage: `BUILD_PROFILE=dev task rust:build` or `task rust:build:dev`

1. **release**: Optimized production builds, minimal debug symbols. Use for production/benchmarking.

   - Cargo profile: release
   - Optimization: -C opt-level=3
   - Usage: `BUILD_PROFILE=release task rust:build` or `task rust:build:release`

1. **ci**: Release optimizations + debug symbols for troubleshooting. Use in CI/CD pipelines.

   - Cargo profile: release
   - Debug: enabled
   - Usage: `BUILD_PROFILE=ci task rust:build` (automatically set in GitHub Actions)

**Profile Mapping**:

- `BUILD_PROFILE=dev` → CARGO_PROFILE_DIR=debug
- `BUILD_PROFILE=release` → CARGO_PROFILE_DIR=release
- `BUILD_PROFILE=ci` → CARGO_PROFILE_DIR=release (but with debug info)

**Usage Examples**:

```bash
# Dev builds (fast, debug symbols)
BUILD_PROFILE=dev task rust:build
BUILD_PROFILE=dev task build:all

# Release builds (optimized)
BUILD_PROFILE=release task rust:build
task build:all:release

# CI builds (optimized + debug)
BUILD_PROFILE=ci task build:all
```
