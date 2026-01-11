//! Language-specific setup functions: building extensions, jars, bindings, etc.

use crate::{Error, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// Ensure PHP extension is built and return its path.
pub fn ensure_php_extension(repo_root: &Path) -> Result<PathBuf> {
    let file_name = platform_library_name("html_to_markdown_php");
    let target_dir = repo_root.join("target").join("release");
    let candidate = target_dir.join(file_name);

    if candidate.exists() {
        return Ok(candidate);
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("-p")
        .arg("html-to-markdown-php")
        .arg("--release")
        .current_dir(repo_root);
    apply_cargo_features(&mut cmd);

    let status = cmd
        .status()
        .map_err(|err| Error::Benchmark(format!("Failed to build php extension: {err}")))?;

    if !status.success() {
        return Err(Error::Benchmark(format!(
            "Failed to build html-to-markdown-php (status: {status})"
        )));
    }

    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(Error::Benchmark(format!(
            "PHP extension not found at {} even after building",
            candidate.display()
        )))
    }
}

/// Ensure Node.js binding is built.
pub fn ensure_node_binding(repo_root: &Path) -> Result<()> {
    let node_dir = repo_root.join("crates/html-to-markdown-node");
    if has_node_binding(&node_dir) {
        return Ok(());
    }

    let mut cmd = Command::new("pnpm");
    cmd.arg("--filter")
        .arg("html-to-markdown-node")
        .arg("run")
        .arg("build")
        .current_dir(repo_root);

    if let Some(flags) = cargo_features_flag() {
        cmd.env("NAPI_RS_CARGO_FLAGS", flags);
    }
    if let Some(features) = cargo_features() {
        cmd.env("NAPI_RS_CARGO_FEATURES", features);
    }

    let status = cmd
        .status()
        .map_err(|err| Error::Benchmark(format!("Failed to build node binding: {err}")))?;

    if status.success() && has_node_binding(&node_dir) {
        Ok(())
    } else {
        Err(Error::Benchmark(format!(
            "Node binding build failed with status {status}"
        )))
    }
}

/// Ensure WASM distribution is built.
pub fn ensure_wasm_dist(repo_root: &Path) -> Result<()> {
    let wasm_dir = repo_root.join("crates/html-to-markdown-wasm");
    let dist_node = wasm_dir.join("dist-node/html_to_markdown_wasm.js");

    if dist_node.exists() {
        return Ok(());
    }

    let status = Command::new("pnpm")
        .arg("--filter")
        .arg("html-to-markdown-wasm")
        .arg("run")
        .arg("build:nodejs")
        .current_dir(repo_root)
        .status()
        .map_err(|err| Error::Benchmark(format!("Failed to build wasm node dist: {err}")))?;

    if status.success() && dist_node.exists() {
        Ok(())
    } else {
        Err(Error::Benchmark(format!("WASM build failed with status {status}")))
    }
}

/// Ensure Java benchmark JAR is built.
pub fn ensure_java_jar(repo_root: &Path) -> Result<()> {
    let java_dir = repo_root.join("packages/java");
    let jar_path = java_dir.join("target/benchmark.jar");

    if jar_path.exists() {
        return Ok(());
    }

    ensure_ffi_library(repo_root)?;

    let mvn_cmd = find_maven_command(repo_root);
    let maven_opts = build_maven_opts()?;
    let release_flag = format!("-Dmaven.compiler.release={}", detect_java_release().unwrap_or(25));

    let maven_skip_args = [
        "-DskipTests",
        "-Dmaven.test.skip=true",
        "-Dgpg.skip=true",
        "-Dmaven.javadoc.skip=true",
    ];

    // Install libraries
    let lib_status = Command::new(&mvn_cmd)
        .arg("install")
        .args(maven_skip_args)
        .arg("-Dskip.rust.ffi=true")
        .arg(&release_flag)
        .env("MAVEN_OPTS", &maven_opts)
        .current_dir(&java_dir)
        .status()
        .map_err(|err| Error::Benchmark(format!("Failed to run mvn package: {err}")))?;

    if !lib_status.success() {
        return Err(Error::Benchmark(format!(
            "Java library build failed with status {lib_status}"
        )));
    }

    // Build benchmark JAR
    let status = Command::new(&mvn_cmd)
        .arg("-f")
        .arg("benchmark-pom.xml")
        .arg("clean")
        .arg("package")
        .args(maven_skip_args)
        .arg("-Dskip.rust.ffi=true")
        .arg(&release_flag)
        .env("MAVEN_OPTS", &maven_opts)
        .current_dir(&java_dir)
        .status()
        .map_err(|err| Error::Benchmark(format!("Failed to build Java benchmark jar: {err}")))?;

    if status.success() && jar_path.exists() {
        Ok(())
    } else {
        Err(Error::Benchmark(format!(
            "Java benchmark JAR build failed with status {status}"
        )))
    }
}

/// Ensure C# benchmark DLL is built.
pub fn ensure_csharp_dll(repo_root: &Path) -> Result<()> {
    ensure_ffi_library(repo_root)?;

    let csharp_dir = repo_root.join("packages/csharp");
    let benchmark_project = csharp_dir.join("Benchmark/Benchmark.csproj");

    if !benchmark_project.exists() {
        return Err(Error::Benchmark(format!(
            "C# benchmark project not found at {}",
            benchmark_project.display()
        )));
    }

    let status = Command::new("dotnet")
        .arg("build")
        .arg("--configuration")
        .arg("Release")
        .arg("Benchmark/Benchmark.csproj")
        .current_dir(&csharp_dir)
        .status()
        .map_err(|err| Error::Benchmark(format!("Failed to build C# benchmark: {err}")))?;

    if !status.success() {
        return Err(Error::Benchmark(format!(
            "C# benchmark build failed with status {status}"
        )));
    }

    copy_ffi_library(repo_root, &csharp_dir.join("Benchmark/bin/Release/net9.0"))
}

/// Ensure Go dependencies are available.
pub fn ensure_go_lib(repo_root: &Path) -> Result<()> {
    ensure_ffi_library(repo_root)?;
    Ok(())
}

/// Ensure Elixir native vendor is set up.
pub fn ensure_elixir_vendor(repo_root: &Path) -> Result<()> {
    let vendor_dir = repo_root.join("packages/elixir/native/html_to_markdown_elixir/vendor");
    let vendor_crate = vendor_dir.join("html-to-markdown-rs");
    let vendor_manifest = vendor_crate.join("Cargo.toml");
    let vendor_workspace = vendor_dir.join("Cargo.toml");

    if vendor_manifest.exists() {
        ensure_elixir_workspace_manifest(repo_root, &vendor_workspace)?;
        return Ok(());
    }

    let source_crate = repo_root.join("crates/html-to-markdown");
    if !source_crate.exists() {
        return Err(Error::Benchmark(format!(
            "Missing Rust crate for Elixir vendor at {}",
            source_crate.display()
        )));
    }

    std::fs::create_dir_all(&vendor_dir)
        .map_err(|err| Error::Benchmark(format!("Failed to create {}: {err}", vendor_dir.display())))?;
    copy_dir_recursive(&source_crate, &vendor_crate)?;
    ensure_elixir_workspace_manifest(repo_root, &vendor_workspace)?;
    Ok(())
}

/// Patch Elixir TOML decoder for compatibility.
pub fn patch_elixir_toml_deps(repo_root: &Path) -> Result<()> {
    let toml_decoder = repo_root.join("packages/elixir/deps/toml/lib/decoder.ex");
    if !toml_decoder.exists() {
        return Ok(());
    }

    let contents = std::fs::read_to_string(&toml_decoder).map_err(|err| {
        Error::Benchmark(format!(
            "Failed to read Elixir toml decoder {}: {err}",
            toml_decoder.display()
        ))
    })?;

    let updated = contents
        .replace("in '-_'", "in ~c\"-_\"")
        .replace("in '-+'", "in ~c\"-+\"")
        .replace("in 'eE'", "in ~c\"eE\"")
        .replace("in '_e.'", "in ~c\"_e.\"")
        .replace("in 'e.'", "in ~c\"e.\"");

    if updated != contents {
        std::fs::write(&toml_decoder, updated).map_err(|err| {
            Error::Benchmark(format!(
                "Failed to write Elixir toml decoder {}: {err}",
                toml_decoder.display()
            ))
        })?;
    }

    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

fn has_node_binding(dir: &Path) -> bool {
    node_binding_candidates().iter().any(|name| dir.join(name).exists())
}

fn node_binding_candidates() -> &'static [&'static str] {
    match (env::consts::OS, env::consts::ARCH) {
        ("macos", "aarch64") => &[
            "html-to-markdown-node.darwin-universal.node",
            "html-to-markdown-node.darwin-arm64.node",
        ],
        ("macos", "x86_64") => &[
            "html-to-markdown-node.darwin-universal.node",
            "html-to-markdown-node.darwin-x64.node",
        ],
        ("linux", "x86_64") => &[
            "html-to-markdown-node.linux-x64-gnu.node",
            "html-to-markdown-node.linux-x64-musl.node",
        ],
        ("linux", "aarch64") => &[
            "html-to-markdown-node.linux-arm64-gnu.node",
            "html-to-markdown-node.linux-arm64-musl.node",
        ],
        ("windows", "x86_64") => &["html-to-markdown-node.win32-x64-msvc.node"],
        ("windows", "aarch64") => &["html-to-markdown-node.win32-arm64-msvc.node"],
        ("windows", "x86") => &["html-to-markdown-node.win32-ia32-msvc.node"],
        _ => &["html-to-markdown-node.node"],
    }
}

fn ensure_ffi_library(repo_root: &Path) -> Result<()> {
    let file_name = platform_library_name("html_to_markdown_ffi");
    let target_dir = repo_root.join("target").join("release");
    let candidate = target_dir.join(&file_name);

    if candidate.exists() && !ffi_needs_rebuild(repo_root, &candidate)? {
        return Ok(());
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("-p")
        .arg("html-to-markdown-ffi")
        .arg("--release")
        .current_dir(repo_root);
    apply_cargo_features(&mut cmd);

    let status = cmd
        .status()
        .map_err(|err| Error::Benchmark(format!("Failed to build FFI library: {err}")))?;

    if !status.success() {
        return Err(Error::Benchmark(format!(
            "Failed to build html-to-markdown-ffi (status: {status})"
        )));
    }

    if candidate.exists() {
        Ok(())
    } else {
        Err(Error::Benchmark(format!(
            "FFI library not found at {} even after building",
            candidate.display()
        )))
    }
}

fn ffi_needs_rebuild(repo_root: &Path, candidate: &Path) -> Result<bool> {
    let lib_meta = std::fs::metadata(candidate)
        .map_err(|err| Error::Benchmark(format!("Failed to read metadata for {}: {err}", candidate.display())))?;
    let lib_modified = lib_meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);

    let ffi_dir = repo_root.join("crates/html-to-markdown-ffi");
    let inputs = [
        ffi_dir.join("Cargo.toml"),
        ffi_dir.join("src/lib.rs"),
        ffi_dir.join("cbindgen.toml"),
    ];

    for path in inputs {
        let meta = std::fs::metadata(&path)
            .map_err(|err| Error::Benchmark(format!("Failed to read metadata for {}: {err}", path.display())))?;
        let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        if modified > lib_modified {
            return Ok(true);
        }
    }

    Ok(false)
}

fn copy_ffi_library(repo_root: &Path, dst_dir: &Path) -> Result<()> {
    let lib_name = platform_library_name("html_to_markdown_ffi");
    let src = repo_root.join("target/release").join(&lib_name);
    let dst = dst_dir.join(&lib_name);

    if !src.exists() {
        return Err(Error::Benchmark(format!(
            "Native library not found at {}",
            src.display()
        )));
    }

    std::fs::create_dir_all(dst_dir)
        .map_err(|err| Error::Benchmark(format!("Failed to create {}: {err}", dst_dir.display())))?;

    std::fs::copy(&src, &dst)
        .map_err(|err| Error::Benchmark(format!("Failed to copy {} to {}: {err}", src.display(), dst.display())))?;

    Ok(())
}

fn ensure_elixir_workspace_manifest(repo_root: &Path, vendor_workspace: &Path) -> Result<()> {
    if vendor_workspace.exists() {
        return Ok(());
    }

    let root_manifest = repo_root.join("Cargo.toml");
    let root_contents = std::fs::read_to_string(&root_manifest)
        .map_err(|err| Error::Benchmark(format!("Failed to read {}: {err}", root_manifest.display())))?;

    let workspace_package = extract_toml_section(&root_contents, "[workspace.package]")
        .ok_or_else(|| Error::Benchmark("Missing [workspace.package] in root Cargo.toml".into()))?;
    let workspace_deps = extract_toml_section(&root_contents, "[workspace.dependencies]")
        .ok_or_else(|| Error::Benchmark("Missing [workspace.dependencies] in root Cargo.toml".into()))?;

    let manifest =
        format!("[workspace]\nmembers = [\"html-to-markdown-rs\"]\n\n{workspace_package}\n\n{workspace_deps}\n");

    std::fs::write(vendor_workspace, manifest).map_err(|err| {
        Error::Benchmark(format!(
            "Failed to write Elixir vendor workspace manifest {}: {err}",
            vendor_workspace.display()
        ))
    })?;
    Ok(())
}

fn extract_toml_section(contents: &str, header: &str) -> Option<String> {
    let start = contents.find(header)?;
    let rest = &contents[start..];
    let mut lines = rest.lines();
    let mut section = Vec::new();

    if let Some(first) = lines.next() {
        section.push(first);
    }
    for line in lines {
        if line.trim_start().starts_with('[') {
            break;
        }
        section.push(line);
    }
    Some(section.join("\n"))
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<()> {
    if destination.exists() {
        std::fs::remove_dir_all(destination)
            .map_err(|err| Error::Benchmark(format!("Failed to remove {}: {err}", destination.display())))?;
    }
    std::fs::create_dir_all(destination)
        .map_err(|err| Error::Benchmark(format!("Failed to create {}: {err}", destination.display())))?;

    for entry in std::fs::read_dir(source)
        .map_err(|err| Error::Benchmark(format!("Failed to read {}: {err}", source.display())))?
    {
        let entry =
            entry.map_err(|err| Error::Benchmark(format!("Failed to read entry in {}: {err}", source.display())))?;
        let entry_path = entry.path();
        let target_path = destination.join(entry.file_name());

        if entry_path.is_dir() {
            copy_dir_recursive(&entry_path, &target_path)?;
        } else {
            std::fs::copy(&entry_path, &target_path).map_err(|err| {
                Error::Benchmark(format!(
                    "Failed to copy {} to {}: {err}",
                    entry_path.display(),
                    target_path.display()
                ))
            })?;
        }
    }
    Ok(())
}

fn platform_library_name(base: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{}.dll", base)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", base)
    } else {
        format!("lib{}.so", base)
    }
}

fn find_maven_command(repo_root: &Path) -> PathBuf {
    if cfg!(windows) {
        let wrapper = repo_root.join("mvnw.cmd");
        if wrapper.exists() {
            wrapper
        } else {
            PathBuf::from("mvn.cmd")
        }
    } else {
        let wrapper = repo_root.join("mvnw");
        if wrapper.exists() {
            wrapper
        } else {
            PathBuf::from("mvn")
        }
    }
}

fn build_maven_opts() -> Result<String> {
    let mut opts = env::var("MAVEN_OPTS").unwrap_or_default();
    let maven_main_class = "org.apache.maven.cli.MavenCli";

    // Add main class
    if !opts.contains("maven.mainClass") {
        if !opts.is_empty() {
            opts.push(' ');
        }
        opts.push_str(&format!("-Dmaven.mainClass={maven_main_class}"));
    }

    // Add native access flag
    if !opts.contains("--enable-native-access") {
        if !opts.is_empty() {
            opts.push(' ');
        }
        opts.push_str("--enable-native-access=ALL-UNNAMED");
    }

    // Add unsafe memory access
    if !opts.contains("sun.misc.unsafe.memory.access") {
        if !opts.is_empty() {
            opts.push(' ');
        }
        opts.push_str("-Dsun.misc.unsafe.memory.access=allow");
    }

    // Add java.base opens
    if !opts.contains("--add-opens=java.base/sun.misc=ALL-UNNAMED") {
        if !opts.is_empty() {
            opts.push(' ');
        }
        opts.push_str("--add-opens=java.base/sun.misc=ALL-UNNAMED");
    }

    Ok(opts)
}

fn detect_java_release() -> Option<u32> {
    let output = Command::new("java").arg("-version").output().ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let text = if stderr.is_empty() { stdout } else { stderr };

    let mut digits = String::new();
    let mut started = false;

    for ch in text.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            started = true;
        } else if started {
            break;
        }
    }

    if digits.is_empty() { None } else { digits.parse().ok() }
}

fn cargo_features() -> Option<String> {
    env::var("HTML_TO_MARKDOWN_CARGO_FEATURES")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn cargo_features_flag() -> Option<String> {
    cargo_features().map(|value| format!("--features {value}"))
}

fn apply_cargo_features(command: &mut Command) {
    if let Some(features) = cargo_features() {
        command.arg("--features").arg(features);
    }
}
