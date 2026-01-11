//! Common utilities for script adapters: command building, profiling, path handling.

use super::language::ScriptLanguage;
use super::setup;
use crate::Result;
use crate::config::{BenchmarkConfig, BenchmarkScenario};
use crate::fixture::Fixture;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Determine the flamegraph output file path and extension based on language.
pub fn flamegraph_path(
    language: &ScriptLanguage,
    fixture: &Fixture,
    scenario: BenchmarkScenario,
    config: &BenchmarkConfig,
) -> Option<PathBuf> {
    if !config.enable_profiling || !supports_profiling() {
        return None;
    }

    let extension = match language {
        ScriptLanguage::Wasm => "html",
        _ => "svg",
    };

    let name = language.as_str();
    config
        .flamegraph_dir
        .as_ref()
        .map(|dir| dir.join(format!("{}-{}-{}.{}", name, fixture.id, scenario.as_str(), extension)))
}

/// Build the command and working directory for a given language.
pub fn build_command(language: &ScriptLanguage, repo_root: &Path) -> Result<(Command, PathBuf)> {
    match language {
        ScriptLanguage::Python => build_python_command(repo_root),
        ScriptLanguage::Ruby => build_ruby_command(repo_root),
        ScriptLanguage::Php => build_php_command(repo_root),
        ScriptLanguage::Node => build_node_command(repo_root),
        ScriptLanguage::Wasm => build_wasm_command(repo_root),
        ScriptLanguage::Java => build_java_command(repo_root),
        ScriptLanguage::CSharp => build_csharp_command(repo_root),
        ScriptLanguage::Go => build_go_command(repo_root),
        ScriptLanguage::Elixir => build_elixir_command(repo_root),
    }
}

const fn supports_profiling() -> bool {
    !cfg!(target_os = "windows")
}

fn build_python_command(repo_root: &Path) -> Result<(Command, PathBuf)> {
    let mut cmd = Command::new("uv");
    cmd.arg("run").arg("python").arg("bin/benchmark.py");
    Ok((cmd, repo_root.join("packages/python")))
}

fn build_ruby_command(repo_root: &Path) -> Result<(Command, PathBuf)> {
    let mut cmd = Command::new("ruby");
    cmd.arg("bin/benchmark.rb");
    cmd.env(
        "RUBYLIB",
        repo_root.join("packages/ruby/lib").to_string_lossy().to_string(),
    );
    Ok((cmd, repo_root.join("packages/ruby")))
}

fn build_php_command(repo_root: &Path) -> Result<(Command, PathBuf)> {
    let extension = setup::ensure_php_extension(repo_root)?;
    let mut cmd = Command::new("php");
    cmd.arg("-d")
        .arg(format!("extension={}", extension.display()))
        .arg(repo_root.join("packages/php/bin/benchmark.php"));
    Ok((cmd, repo_root.to_path_buf()))
}

fn build_node_command(repo_root: &Path) -> Result<(Command, PathBuf)> {
    setup::ensure_node_binding(repo_root)?;
    let mut cmd = Command::new("pnpm");
    cmd.arg("-C")
        .arg("crates/html-to-markdown-node")
        .arg("exec")
        .arg("tsx")
        .arg("bin/benchmark.ts");
    Ok((cmd, repo_root.to_path_buf()))
}

fn build_wasm_command(repo_root: &Path) -> Result<(Command, PathBuf)> {
    setup::ensure_wasm_dist(repo_root)?;
    let mut cmd = Command::new("pnpm");
    cmd.arg("--filter")
        .arg("html-to-markdown-wasm")
        .arg("exec")
        .arg("tsx")
        .arg("bin/benchmark.ts");
    Ok((cmd, repo_root.to_path_buf()))
}

fn build_java_command(repo_root: &Path) -> Result<(Command, PathBuf)> {
    setup::ensure_java_jar(repo_root)?;
    let mut cmd = Command::new("java");
    cmd.arg("--enable-preview")
        .arg("--enable-native-access=ALL-UNNAMED")
        .arg(format!(
            "-Djava.library.path={}",
            repo_root.join("target/release").display()
        ))
        .arg("-jar")
        .arg("target/benchmark.jar");
    Ok((cmd, repo_root.join("packages/java")))
}

fn build_csharp_command(repo_root: &Path) -> Result<(Command, PathBuf)> {
    setup::ensure_csharp_dll(repo_root)?;
    let mut cmd = Command::new("dotnet");
    cmd.arg("run")
        .arg("--configuration")
        .arg("Release")
        .arg("--project")
        .arg("Benchmark/Benchmark.csproj")
        .arg("--");

    let lib_dir = repo_root.join("target/release");
    append_library_path(&mut cmd, "LD_LIBRARY_PATH", &lib_dir);
    append_library_path(&mut cmd, "DYLD_LIBRARY_PATH", &lib_dir);

    Ok((cmd, repo_root.join("packages/csharp")))
}

fn build_go_command(repo_root: &Path) -> Result<(Command, PathBuf)> {
    setup::ensure_go_lib(repo_root)?;
    let mut cmd = Command::new("go");
    cmd.arg("run").arg("bin/benchmark.go");

    let lib_dir = repo_root.join("target/release").to_string_lossy().to_string();
    cmd.env("CGO_LDFLAGS", format!("-L{}", lib_dir));
    cmd.env("GODEBUG", "cgocheck=0");

    append_library_path(&mut cmd, "LD_LIBRARY_PATH", &PathBuf::from(&lib_dir));
    append_library_path(&mut cmd, "DYLD_LIBRARY_PATH", &PathBuf::from(&lib_dir));

    Ok((cmd, repo_root.join("packages/go/v2")))
}

fn build_elixir_command(repo_root: &Path) -> Result<(Command, PathBuf)> {
    setup::ensure_elixir_vendor(repo_root)?;
    setup::patch_elixir_toml_deps(repo_root)?;
    let mut cmd = Command::new("mix");
    cmd.arg("run").arg("scripts/benchmark.exs");
    cmd.env("MIX_ENV", "prod");
    cmd.env("MIX_QUIET", "1");
    Ok((cmd, repo_root.join("packages/elixir")))
}

/// Helper to append a library path to an environment variable, preserving existing values.
fn append_library_path(cmd: &mut Command, var_name: &str, lib_path: &Path) {
    let mut path = lib_path.to_string_lossy().to_string();
    if let Ok(existing) = env::var(var_name) {
        path = format!("{}:{}", path, existing);
    }
    cmd.env(var_name, path);
}
