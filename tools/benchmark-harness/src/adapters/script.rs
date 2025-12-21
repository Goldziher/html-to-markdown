use crate::adapter::FrameworkAdapter;
use crate::config::{BenchmarkConfig, BenchmarkScenario};
use crate::fixture::{Fixture, FixtureFormat};
use crate::monitoring::ResourceMonitor;
use crate::types::{BenchmarkResult, PerformanceMetrics};
use crate::{Error, Result};
use serde::Deserialize;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLanguage {
    Python,
    Ruby,
    Php,
    Node,
    Wasm,
    Java,
    CSharp,
    Go,
    Elixir,
}

impl ScriptLanguage {
    fn as_str(&self) -> &'static str {
        match self {
            ScriptLanguage::Python => "python",
            ScriptLanguage::Ruby => "ruby",
            ScriptLanguage::Php => "php",
            ScriptLanguage::Node => "node",
            ScriptLanguage::Wasm => "wasm",
            ScriptLanguage::Java => "java",
            ScriptLanguage::CSharp => "csharp",
            ScriptLanguage::Go => "go",
            ScriptLanguage::Elixir => "elixir",
        }
    }
}

pub struct ScriptAdapter {
    language: ScriptLanguage,
    repo_root: PathBuf,
}

impl ScriptAdapter {
    pub fn new(language: ScriptLanguage, repo_root: PathBuf) -> Self {
        Self { language, repo_root }
    }

    fn supports_profiling(&self) -> bool {
        if cfg!(target_os = "windows") {
            return false;
        }
        true
    }

    fn flamegraph_path(
        &self,
        fixture: &Fixture,
        scenario: BenchmarkScenario,
        config: &BenchmarkConfig,
    ) -> Option<PathBuf> {
        if config.enable_profiling && self.supports_profiling() {
            let extension = match self.language {
                ScriptLanguage::Wasm => "html",
                _ => "svg",
            };
            config.flamegraph_dir.as_ref().map(|dir| {
                dir.join(format!(
                    "{}-{}-{}.{}",
                    self.name(),
                    fixture.id,
                    scenario.as_str(),
                    extension
                ))
            })
        } else {
            None
        }
    }

    fn build_command(&self) -> Result<(Command, PathBuf)> {
        match self.language {
            ScriptLanguage::Python => {
                let mut cmd = Command::new("uv");
                cmd.arg("run").arg("python").arg("bin/benchmark.py");
                Ok((cmd, self.repo_root.join("packages/python")))
            }
            ScriptLanguage::Ruby => {
                let mut cmd = Command::new("ruby");
                cmd.arg("bin/benchmark.rb");
                cmd.env(
                    "RUBYLIB",
                    self.repo_root.join("packages/ruby/lib").to_string_lossy().to_string(),
                );
                Ok((cmd, self.repo_root.join("packages/ruby")))
            }
            ScriptLanguage::Php => {
                let extension = ensure_php_extension(&self.repo_root)?;
                let mut cmd = Command::new("php");
                cmd.arg("-d")
                    .arg(format!("extension={}", extension.display()))
                    .arg(self.repo_root.join("packages/php/bin/benchmark.php"));
                Ok((cmd, self.repo_root.to_path_buf()))
            }
            ScriptLanguage::Node => {
                ensure_node_binding(&self.repo_root)?;
                let mut cmd = Command::new("pnpm");
                cmd.arg("--filter")
                    .arg("html-to-markdown-node")
                    .arg("exec")
                    .arg("tsx")
                    .arg("crates/html-to-markdown-node/bin/benchmark.ts");
                Ok((cmd, self.repo_root.to_path_buf()))
            }
            ScriptLanguage::Wasm => {
                ensure_wasm_dist(&self.repo_root)?;
                let mut cmd = Command::new("pnpm");
                cmd.arg("--filter")
                    .arg("html-to-markdown-wasm")
                    .arg("exec")
                    .arg("tsx")
                    .arg("bin/benchmark.ts");
                Ok((cmd, self.repo_root.to_path_buf()))
            }
            ScriptLanguage::Java => {
                ensure_java_jar(&self.repo_root)?;
                let mut cmd = Command::new("java");
                cmd.arg("--enable-preview")
                    .arg("--enable-native-access=ALL-UNNAMED")
                    .arg(format!(
                        "-Djava.library.path={}",
                        self.repo_root.join("target/release").display()
                    ))
                    .arg("-jar")
                    .arg("target/benchmark.jar");
                Ok((cmd, self.repo_root.join("packages/java")))
            }
            ScriptLanguage::CSharp => {
                ensure_csharp_dll(&self.repo_root)?;
                let mut cmd = Command::new("dotnet");
                cmd.arg("run")
                    .arg("--configuration")
                    .arg("Release")
                    .arg("--project")
                    .arg("Benchmark/Benchmark.csproj")
                    .arg("--");
                let mut ld_path = self.repo_root.join("target/release").to_string_lossy().to_string();
                if let Ok(existing) = env::var("LD_LIBRARY_PATH") {
                    ld_path = format!("{}:{}", ld_path, existing);
                }
                cmd.env("LD_LIBRARY_PATH", &ld_path);
                let mut dyld_path = self.repo_root.join("target/release").to_string_lossy().to_string();
                if let Ok(existing) = env::var("DYLD_LIBRARY_PATH") {
                    dyld_path = format!("{}:{}", dyld_path, existing);
                }
                cmd.env("DYLD_LIBRARY_PATH", &dyld_path);
                Ok((cmd, self.repo_root.join("packages/csharp")))
            }
            ScriptLanguage::Go => {
                ensure_go_lib(&self.repo_root)?;
                let mut cmd = Command::new("go");
                cmd.arg("run").arg("bin/benchmark.go");
                let lib_dir = self.repo_root.join("target/release").to_string_lossy().to_string();
                cmd.env("CGO_LDFLAGS", format!("-L{}", lib_dir));
                cmd.env("GODEBUG", "cgocheck=0");

                let mut ld_path = lib_dir.clone();
                if let Ok(existing) = env::var("LD_LIBRARY_PATH") {
                    ld_path = format!("{}:{}", ld_path, existing);
                }
                cmd.env("LD_LIBRARY_PATH", &ld_path);

                let mut dyld_path = lib_dir.clone();
                if let Ok(existing) = env::var("DYLD_LIBRARY_PATH") {
                    dyld_path = format!("{}:{}", dyld_path, existing);
                }
                cmd.env("DYLD_LIBRARY_PATH", &dyld_path);
                Ok((cmd, self.repo_root.join("packages/go/v2")))
            }
            ScriptLanguage::Elixir => {
                ensure_elixir_vendor(&self.repo_root)?;
                patch_elixir_toml_deps(&self.repo_root)?;
                let mut cmd = Command::new("mix");
                cmd.arg("run").arg("scripts/benchmark.exs");
                cmd.env("MIX_ENV", "prod");
                cmd.env("MIX_QUIET", "1");
                Ok((cmd, self.repo_root.join("packages/elixir")))
            }
        }
    }

    fn run_wasm_profile(
        &self,
        fixture: &Fixture,
        scenario: BenchmarkScenario,
        config: &BenchmarkConfig,
        output: &Path,
    ) -> Result<()> {
        let output_dir = output.parent().unwrap_or(&config.output_dir).join(format!(
            "wasm-profile-{}-{}",
            fixture.id,
            scenario.as_str()
        ));
        std::fs::create_dir_all(&output_dir)
            .map_err(|err| Error::Benchmark(format!("Failed to create {}: {err}", output_dir.display())))?;

        let iterations = fixture.iterations.unwrap_or(config.benchmark_iterations as u32).max(1) as usize;
        let fixture_path = fixture.resolved_path(&self.repo_root);

        let mut cmd = Command::new("pnpm");
        cmd.arg("--filter")
            .arg("html-to-markdown-wasm")
            .arg("exec")
            .arg("0x")
            .arg("--output-dir")
            .arg(&output_dir)
            .arg("--")
            .arg("node")
            .arg("--import")
            .arg("tsx")
            .arg("bin/benchmark.ts")
            .arg("--file")
            .arg(&fixture_path)
            .arg("--iterations")
            .arg(iterations.to_string())
            .arg("--scenario")
            .arg(scenario.as_str())
            .arg("--format")
            .arg(match fixture.format {
                FixtureFormat::Html => "html",
                FixtureFormat::Hocr => "hocr",
            });

        cmd.env("HTML_TO_MARKDOWN_BENCH_WARMUP", config.warmup_iterations.to_string());
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
        cmd.current_dir(&self.repo_root);

        let status = cmd
            .status()
            .map_err(|err| Error::Benchmark(format!("Failed to run wasm profiler: {err}")))?;
        if !status.success() {
            return Err(Error::Benchmark(format!("Wasm profiling failed with status {status}")));
        }

        let flamegraph = output_dir.join("flamegraph.html");
        if !flamegraph.exists() {
            return Err(Error::Benchmark(format!(
                "Wasm profiling did not produce {}",
                flamegraph.display()
            )));
        }

        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|err| Error::Benchmark(format!("Failed to create {}: {err}", parent.display())))?;
        }

        std::fs::copy(&flamegraph, output).map_err(|err| {
            Error::Benchmark(format!(
                "Failed to copy wasm flamegraph {} to {}: {err}",
                flamegraph.display(),
                output.display()
            ))
        })?;

        Ok(())
    }
}

impl FrameworkAdapter for ScriptAdapter {
    fn name(&self) -> &str {
        self.language.as_str()
    }

    fn supports_format(&self, format: FixtureFormat) -> bool {
        matches!(format, FixtureFormat::Html | FixtureFormat::Hocr)
    }

    fn supports_scenario(&self, scenario: BenchmarkScenario) -> bool {
        match self.language {
            ScriptLanguage::Java | ScriptLanguage::CSharp | ScriptLanguage::Go => matches!(
                scenario,
                BenchmarkScenario::ConvertDefault | BenchmarkScenario::MetadataDefault
            ),
            _ => true,
        }
    }

    fn run(&self, fixture: &Fixture, scenario: BenchmarkScenario, config: &BenchmarkConfig) -> Result<BenchmarkResult> {
        let (mut command, working_dir) = self.build_command()?;
        let fixture_path = fixture.resolved_path(&self.repo_root);
        let flamegraph_path = self.flamegraph_path(fixture, scenario, config);
        let flamegraph_output_path = flamegraph_path.as_ref().map(|path| {
            if path.is_relative() {
                self.repo_root.join(path)
            } else {
                path.clone()
            }
        });
        let flamegraph_result_path = flamegraph_path.as_ref().map(|path| {
            if let Ok(relative) = path.strip_prefix(&config.output_dir) {
                relative.to_path_buf()
            } else {
                path.clone()
            }
        });

        let iterations = fixture.iterations.unwrap_or(config.benchmark_iterations as u32).max(1) as usize;

        if let Some(output) = flamegraph_output_path.as_ref() {
            if matches!(self.language, ScriptLanguage::Wasm) {
                self.run_wasm_profile(fixture, scenario, config, output)?;
            } else {
                command.env("HTML_TO_MARKDOWN_PROFILE_OUTPUT", output);
                command.env("HTML_TO_MARKDOWN_PROFILE_ONCE", "true");
                command.env(
                    "HTML_TO_MARKDOWN_PROFILE_FREQUENCY",
                    config.profile_frequency.to_string(),
                );
                let repeat = if matches!(self.language, ScriptLanguage::Elixir) {
                    config.profile_repeat.max(5)
                } else {
                    config.profile_repeat
                };
                command.env("HTML_TO_MARKDOWN_PROFILE_REPEAT", repeat.to_string());
            }
        }

        command.env("HTML_TO_MARKDOWN_BENCH_WARMUP", config.warmup_iterations.to_string());
        command.env("HTML_TO_MARKDOWN_FAST_FFI", "1");

        command
            .arg("--file")
            .arg(&fixture_path)
            .arg("--iterations")
            .arg(iterations.to_string())
            .arg("--scenario")
            .arg(scenario.as_str())
            .arg("--format")
            .arg(fixture.format.as_str())
            .current_dir(&working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());

        let child = command
            .spawn()
            .map_err(|err| Error::Benchmark(format!("Failed to run {} benchmark: {err}", self.name())))?;

        let monitor = ResourceMonitor::start(child.id(), Duration::from_millis(config.sample_interval_ms));
        let output = child.wait_with_output().map_err(Error::Io)?;
        let resource_stats = monitor.map(|m| m.stop()).unwrap_or_default();

        if !output.status.success() {
            let stderr_tail = tail_lines(&String::from_utf8_lossy(&output.stderr), 5);
            let error_message = if stderr_tail.is_empty() {
                format!("{} exited with status {}", self.name(), output.status)
            } else {
                format!(
                    "{} exited with status {} (stderr tail: {})",
                    self.name(),
                    output.status,
                    stderr_tail
                )
            };
            return Ok(BenchmarkResult {
                framework: self.name().to_string(),
                scenario: scenario.as_str().to_string(),
                fixture_id: fixture.id.clone(),
                fixture_name: fixture.name.clone(),
                fixture_path: fixture_path.clone(),
                fixture_format: fixture.format.as_str().to_string(),
                file_extension: fixture.file_extension(),
                file_size: std::fs::metadata(&fixture_path).map(|m| m.len()).unwrap_or_default(),
                iterations,
                duration: Duration::from_secs(0),
                metrics: PerformanceMetrics::default(),
                resource_stats,
                memory_stats: None,
                flamegraph_path: None,
                statistics: None,
                success: false,
                error_message: Some(error_message),
            });
        }

        let stdout = String::from_utf8(output.stdout)
            .map_err(|err| Error::Benchmark(format!("Invalid UTF-8 from {}: {err}", self.name())))?;
        let script_result: ScriptResult = match serde_json::from_str(stdout.trim()) {
            Ok(result) => result,
            Err(err) => {
                if let Some(line) = extract_json_line(&stdout) {
                    serde_json::from_str(&line)
                        .map_err(|err| Error::Benchmark(format!("Failed to parse {} output: {err}", self.name())))?
                } else {
                    let stdout_tail = tail_lines(&stdout, 5);
                    let stderr_tail = tail_lines(&String::from_utf8_lossy(&output.stderr), 5);
                    let mut context = String::new();
                    if !stdout_tail.is_empty() {
                        context.push_str(&format!(" stdout tail: {}", stdout_tail));
                    }
                    if !stderr_tail.is_empty() {
                        context.push_str(&format!(" stderr tail: {}", stderr_tail));
                    }
                    return Err(Error::Benchmark(format!(
                        "Failed to parse {} output: {err}.{}",
                        self.name(),
                        context
                    )));
                }
            }
        };

        let duration = Duration::from_secs_f64(script_result.elapsed_seconds);
        let file_size = std::fs::metadata(&fixture_path).map(|m| m.len()).unwrap_or_default();
        let profile_repeat = if flamegraph_output_path.is_some() {
            config.profile_repeat.max(1)
        } else {
            1
        };
        let iterations = (script_result.iterations as usize).saturating_mul(profile_repeat);
        let bytes_processed = (script_result.bytes_processed as u64).saturating_mul(profile_repeat as u64);
        let duration_secs = duration.as_secs_f64().max(0.000_001);
        let ops_per_sec = iterations as f64 / duration_secs;
        let mb_per_sec = (bytes_processed as f64 / (1024.0 * 1024.0)) / duration_secs;

        Ok(BenchmarkResult {
            framework: self.name().to_string(),
            scenario: script_result.scenario.unwrap_or_else(|| scenario.as_str().to_string()),
            fixture_id: fixture.id.clone(),
            fixture_name: fixture.name.clone(),
            fixture_path: fixture_path.clone(),
            fixture_format: fixture.format.as_str().to_string(),
            file_extension: fixture.file_extension(),
            file_size,
            iterations,
            duration,
            metrics: PerformanceMetrics {
                ops_per_sec,
                mb_per_sec,
                throughput_bytes_per_sec: if duration.as_secs_f64() > 0.0 {
                    bytes_processed as f64 / duration.as_secs_f64()
                } else {
                    0.0
                },
            },
            resource_stats,
            memory_stats: None,
            flamegraph_path: flamegraph_result_path,
            statistics: None,
            success: true,
            error_message: None,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ScriptResult {
    language: String,
    fixture: String,
    #[serde(default)]
    fixture_path: Option<PathBuf>,
    #[serde(default)]
    scenario: Option<String>,
    iterations: u32,
    elapsed_seconds: f64,
    ops_per_sec: f64,
    mb_per_sec: f64,
    bytes_processed: usize,
}

fn ensure_php_extension(repo_root: &Path) -> Result<PathBuf> {
    let file_name = if cfg!(target_os = "windows") {
        "html_to_markdown_php.dll"
    } else if cfg!(target_os = "macos") {
        "libhtml_to_markdown_php.dylib"
    } else {
        "libhtml_to_markdown_php.so"
    };

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

fn ensure_node_binding(repo_root: &Path) -> Result<()> {
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

fn ensure_wasm_dist(repo_root: &Path) -> Result<()> {
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

fn ensure_java_jar(repo_root: &Path) -> Result<()> {
    let java_dir = repo_root.join("packages/java");
    let jar_path = java_dir.join("target/benchmark.jar");

    if jar_path.exists() {
        return Ok(());
    }

    ensure_ffi_library(repo_root)?;

    let mvn_cmd = if cfg!(windows) {
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
    };
    let maven_main_class = "org.apache.maven.cling.MavenCling";
    let mut maven_opts = env::var("MAVEN_OPTS").unwrap_or_default();
    if !maven_opts.contains("maven.mainClass") {
        if !maven_opts.is_empty() {
            maven_opts.push(' ');
        }
        maven_opts.push_str(&format!("-Dmaven.mainClass={maven_main_class}"));
    }
    if !maven_opts.contains("--enable-native-access") {
        if !maven_opts.is_empty() {
            maven_opts.push(' ');
        }
        maven_opts.push_str("--enable-native-access=ALL-UNNAMED");
    }
    if !maven_opts.contains("--add-opens=java.base/sun.misc=ALL-UNNAMED") {
        if !maven_opts.is_empty() {
            maven_opts.push(' ');
        }
        maven_opts.push_str("--add-opens=java.base/sun.misc=ALL-UNNAMED");
    }

    let maven_skip_args = [
        "-DskipTests",
        "-Dmaven.test.skip=true",
        "-Dgpg.skip=true",
        "-Dmaven.javadoc.skip=true",
    ];
    let java_release = env::var("HTML_TO_MARKDOWN_JAVA_RELEASE")
        .ok()
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or(25);
    if let Some(version) = detect_java_release() {
        if version < java_release {
            return Err(Error::Benchmark(format!(
                "Java {java_release}+ required for benchmarks (found {version})."
            )));
        }
    }
    let release_flag = format!("-Dmaven.compiler.release={java_release}");

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

fn ensure_csharp_dll(repo_root: &Path) -> Result<()> {
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

    let lib_dir = repo_root.join("target/release");
    let benchmark_output = csharp_dir.join("Benchmark/bin/Release/net9.0");

    #[cfg(target_os = "macos")]
    let lib_name = "libhtml_to_markdown_ffi.dylib";
    #[cfg(target_os = "linux")]
    let lib_name = "libhtml_to_markdown_ffi.so";
    #[cfg(target_os = "windows")]
    let lib_name = "html_to_markdown_ffi.dll";

    let src = lib_dir.join(lib_name);
    let dst = benchmark_output.join(lib_name);

    if !src.exists() {
        return Err(Error::Benchmark(format!(
            "Native library not found at {}",
            src.display()
        )));
    }

    std::fs::create_dir_all(&benchmark_output)
        .map_err(|err| Error::Benchmark(format!("Failed to create {}: {err}", benchmark_output.display())))?;

    std::fs::copy(&src, &dst)
        .map_err(|err| Error::Benchmark(format!("Failed to copy {} to {}: {err}", src.display(), dst.display())))?;

    Ok(())
}

fn ensure_go_lib(repo_root: &Path) -> Result<()> {
    ensure_ffi_library(repo_root)?;
    Ok(())
}

fn ensure_elixir_vendor(repo_root: &Path) -> Result<()> {
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

fn patch_elixir_toml_deps(repo_root: &Path) -> Result<()> {
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

fn extract_json_line(output: &str) -> Option<String> {
    for line in output.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn tail_lines(output: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .rev()
        .take(max_lines)
        .collect();
    if lines.is_empty() {
        return String::new();
    }
    lines.into_iter().rev().collect::<Vec<_>>().join(" | ")
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

fn ensure_ffi_library(repo_root: &Path) -> Result<()> {
    let file_name = if cfg!(target_os = "windows") {
        "html_to_markdown_ffi.dll"
    } else if cfg!(target_os = "macos") {
        "libhtml_to_markdown_ffi.dylib"
    } else {
        "libhtml_to_markdown_ffi.so"
    };

    let target_dir = repo_root.join("target").join("release");
    let candidate = target_dir.join(file_name);

    if candidate.exists() {
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
