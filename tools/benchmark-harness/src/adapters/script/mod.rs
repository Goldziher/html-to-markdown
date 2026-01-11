mod common;
mod language;
mod setup;

use crate::adapter::FrameworkAdapter;
use crate::config::{BenchmarkConfig, BenchmarkScenario};
use crate::fixture::{Fixture, FixtureFormat};
use crate::monitoring::ResourceMonitor;
use crate::types::{BenchmarkResult, PerformanceMetrics};
use crate::{Error, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

pub use language::ScriptLanguage;

pub struct ScriptAdapter {
    language: ScriptLanguage,
    repo_root: PathBuf,
}

impl ScriptAdapter {
    pub const fn new(language: ScriptLanguage, repo_root: PathBuf) -> Self {
        Self { language, repo_root }
    }

    fn flamegraph_path(
        &self,
        fixture: &Fixture,
        scenario: BenchmarkScenario,
        config: &BenchmarkConfig,
    ) -> Option<PathBuf> {
        common::flamegraph_path(&self.language, fixture, scenario, config)
    }

    fn build_command(&self) -> Result<(Command, PathBuf)> {
        common::build_command(&self.language, &self.repo_root)
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
        if matches!(scenario, BenchmarkScenario::MetadataRaw) {
            return matches!(self.language, ScriptLanguage::CSharp);
        }

        match self.language {
            ScriptLanguage::Java | ScriptLanguage::Go => matches!(
                scenario,
                BenchmarkScenario::ConvertDefault | BenchmarkScenario::MetadataDefault
            ),
            ScriptLanguage::CSharp => matches!(
                scenario,
                BenchmarkScenario::ConvertDefault | BenchmarkScenario::MetadataDefault | BenchmarkScenario::MetadataRaw
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
            .arg(fixture.format.as_str());

        if fixture.visitor.as_str() != "none" {
            command.arg("--visitor").arg(fixture.visitor.as_str());
        }

        command
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
                framework: config.framework_label(self.name()),
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
        let iterations = script_result.iterations as usize;
        let bytes_processed = script_result.bytes_processed as u64;
        let duration_secs = duration.as_secs_f64().max(0.000_001);
        let ops_per_sec = iterations as f64 / duration_secs;
        let mb_per_sec = (bytes_processed as f64 / (1024.0 * 1024.0)) / duration_secs;

        Ok(BenchmarkResult {
            framework: config.framework_label(self.name()),
            scenario: script_result.scenario.unwrap_or_else(|| scenario.as_str().to_string()),
            fixture_id: fixture.id.clone(),
            fixture_name: fixture.name.clone(),
            fixture_path,
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
