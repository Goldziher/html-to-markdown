______________________________________________________________________

## priority: critical

# Platform Support & Cross-Platform Compatibility

**Supported Platforms**: Windows (via Git Bash/MSYS2/MinGW/PowerShell), Linux (all major distributions), macOS (Intel & ARM/M-series).

**Platform Detection in Taskfiles**:

- `{{.OS}}`: Detects operating system (windows, linux, darwin)
  - Windows detection: PowerShell, GOOS env, WINDIR/SYSTEMROOT, MSYSTEM (Git Bash/MSYS2/MinGW)
  - Linux/macOS: uname fallback with comprehensive error handling
- `{{.ARCH}}`: Detects CPU architecture (x86_64, arm64, etc.)
- `{{.EXE_EXT}}`: Platform-specific executable extension (.exe on Windows, empty on Unix)
- `{{.LIB_EXT}}`: Platform-specific library extension (dll on Windows, dylib on macOS, so on Linux)
- `{{.NUM_CPUS}}`: CPU count for parallel builds (PowerShell/sysctl/nproc/cpuinfo detection)

**Platform Guards in Task Files**:

- Use conditional commands: `cmd: "script.sh"; platforms: [linux, darwin]`
- Windows-specific: `cmd: "script.bat"; platforms: [windows]`
- Darwin-specific: `cmd: "script.sh"; platforms: [darwin]`
- Example: Cargo commands work on all platforms (Rust toolchain cross-platform)

**Cross-Platform Best Practices**:

1. Use Taskfile variables instead of hardcoded paths ({{.ROOT}}, {{.CRATES_DIR}})
1. Use forward slashes in paths; Taskfile converts to backslashes on Windows
1. Avoid shell-specific features; use platform-agnostic task commands
1. For scripts: create separate .sh (Unix) and .bat/.ps1 (Windows) files if needed
1. Test on Windows, Linux, and macOS in CI (or locally via Docker)

**Environment Variables for Cross-Platform**:

- `LD_LIBRARY_PATH` (Linux): Add target/release/target/debug for FFI tests
- `DYLD_LIBRARY_PATH` (macOS): Add target/release/target/debug for FFI tests
- `PATH` (Windows): Automatically uses backslashes for library lookup
