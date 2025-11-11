use std::{
    collections::BTreeMap,
    env, fmt, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use clap::{ArgAction, Parser, ValueEnum};
use humansize::{BINARY, format_size_i};
#[cfg(not(target_os = "windows"))]
use pprof::ProfilerGuardBuilder;
use serde::{Deserialize, Serialize};

mod monitor;
use monitor::{ResourceMonitor, ResourceStats};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Benchmark non-Rust bindings (PHP/Ruby) using shared fixtures"
)]
struct Args {
    /// Languages to benchmark (repeat flag). Defaults to both.
    #[arg(long = "language", value_enum, action = ArgAction::Append)]
    languages: Vec<Language>,

    /// Fixture definition file (TOML).
    #[arg(
        long,
        default_value = "tools/runtime-bench/fixtures/wikipedia.toml",
        value_name = "FILE"
    )]
    fixtures: PathBuf,

    /// Limit to specific fixture IDs (repeat flag).
    #[arg(long = "fixture", action = ArgAction::Append, value_name = "ID")]
    fixture_ids: Vec<String>,

    /// Override iteration count for fixtures that do not specify one.
    #[arg(long, default_value_t = 50)]
    iterations: u32,

    /// Optional JSON output path.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Collect per-process CPU/memory samples while benchmarks run.
    #[arg(long)]
    profile: bool,

    /// Generate a flamegraph (Rust language only).
    #[arg(long, value_name = "FILE")]
    flamegraph: Option<PathBuf>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, ValueEnum, Serialize, Deserialize)]
enum Language {
    Php,
    Ruby,
    Python,
    Node,
    Wasm,
    Rust,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Php => write!(f, "PHP"),
            Language::Ruby => write!(f, "Ruby"),
            Language::Python => write!(f, "Python"),
            Language::Node => write!(f, "Node"),
            Language::Wasm => write!(f, "WASM"),
            Language::Rust => write!(f, "Rust"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct FixtureFile {
    fixtures: Vec<Fixture>,
}

#[derive(Debug, Deserialize)]
struct Fixture {
    id: String,
    name: String,
    path: PathBuf,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    iterations: Option<u32>,
    #[serde(default)]
    format: FixtureFormat,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ScriptResult {
    language: String,
    fixture: String,
    #[serde(default)]
    fixture_path: Option<PathBuf>,
    iterations: u32,
    elapsed_seconds: f64,
    ops_per_sec: f64,
    mb_per_sec: f64,
    bytes_processed: usize,
}

#[derive(Debug, Serialize)]
struct HarnessResult {
    language: Language,
    fixture_id: String,
    fixture_name: String,
    category: Option<String>,
    format: FixtureFormat,
    iterations: u32,
    elapsed_seconds: f64,
    ops_per_sec: f64,
    mb_per_sec: f64,
    bytes_processed: usize,
    payload_size_bytes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_stats: Option<ResourceStats>,
}

impl HarnessResult {
    fn table_row(&self) -> String {
        let (peak_mem, avg_cpu) = if let Some(stats) = &self.resource_stats {
            (
                format_size_i(stats.peak_memory_bytes as i64, BINARY),
                format!("{:.1}%", stats.avg_cpu_percent),
            )
        } else {
            ("-".to_string(), "-".to_string())
        };

        format!(
            "| {fixture:<22} | {language:<6} | {fmt:<4} | {ops:>10.0} | {mb:>9.1} | {bytes:<12} | {iters:>5} | {mem:>10} | {cpu:>7} |",
            fixture = self.fixture_name,
            language = self.language,
            fmt = self.format.as_str(),
            ops = self.ops_per_sec,
            mb = self.mb_per_sec,
            bytes = format_size_i(self.payload_size_bytes as i64, BINARY),
            iters = self.iterations,
            mem = peak_mem,
            cpu = avg_cpu,
        )
    }
}

fn main() -> Result<()> {
    let mut args = Args::parse();
    if cfg!(debug_assertions) {
        eprintln!(
            "Warning: runtime bench is running in debug mode; use `cargo run --release ...` for production-accurate Rust numbers."
        );
    }
    if args.languages.is_empty() {
        args.languages = vec![
            Language::Php,
            Language::Ruby,
            Language::Python,
            Language::Node,
            Language::Wasm,
            Language::Rust,
        ];
    }

    if args.flamegraph.is_some() && (args.languages.len() != 1 || args.languages[0] != Language::Rust) {
        bail!("--flamegraph is only supported when benchmarking Rust alone (use --language rust).");
    }

    let fixture_file = fs::read_to_string(&args.fixtures)
        .with_context(|| format!("Failed to read fixture file {}", args.fixtures.display()))?;
    let fixtures: FixtureFile = toml::from_str(&fixture_file)
        .with_context(|| format!("Failed to parse fixture file {}", args.fixtures.display()))?;

    let selected: Vec<_> = fixtures
        .fixtures
        .iter()
        .filter(|fixture| {
            if args.fixture_ids.is_empty() {
                true
            } else {
                args.fixture_ids.iter().any(|id| id == &fixture.id)
            }
        })
        .collect();

    if selected.is_empty() {
        bail!("No fixtures selected. Provide --fixture ID to target specific fixtures.");
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .context("Unable to determine repository root")?;

    let mut results = Vec::new();

    for language in &args.languages {
        for fixture in &selected {
            let payload_path = repo_root.join(&fixture.path);
            if !payload_path.exists() {
                bail!("Fixture {} not found at {}", fixture.id, payload_path.display());
            }

            let html = fs::read(&payload_path)
                .with_context(|| format!("Failed to read HTML fixture {}", payload_path.display()))?;

            let iterations = fixture.iterations.unwrap_or(args.iterations);

            let (script_result, resource_stats) = if *language == Language::Rust {
                let html_str = String::from_utf8(html.clone())
                    .with_context(|| format!("Fixture {} is not valid UTF-8", payload_path.display()))?;
                run_rust_benchmark(
                    &payload_path,
                    &html_str,
                    iterations,
                    fixture.format,
                    args.flamegraph.as_deref(),
                )
                .with_context(|| format!("Benchmark failed for {} ({})", fixture.id, language))?
            } else {
                run_script(
                    language,
                    &payload_path,
                    iterations,
                    &repo_root,
                    args.profile,
                    fixture.format,
                )
                .with_context(|| format!("Benchmark failed for {} ({})", fixture.id, language))?
            };

            let harness_result = HarnessResult {
                language: *language,
                fixture_id: fixture.id.clone(),
                fixture_name: fixture.name.clone(),
                category: fixture.category.clone(),
                format: fixture.format,
                iterations: script_result.iterations,
                elapsed_seconds: script_result.elapsed_seconds,
                ops_per_sec: script_result.ops_per_sec,
                mb_per_sec: script_result.mb_per_sec,
                bytes_processed: script_result.bytes_processed,
                payload_size_bytes: html.len(),
                resource_stats,
            };

            results.push(harness_result);
        }
    }

    print_summary(&results);

    if let Some(path) = args.output {
        let parent = path.parent().context("Invalid output path")?;
        if !parent.exists() {
            fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(&results)?;
        fs::write(&path, json).with_context(|| format!("Failed to write {}", path.display()))?;
        println!("\nResults saved to {}", path.display());
    }

    Ok(())
}

fn run_script(
    language: &Language,
    file: &Path,
    iterations: u32,
    repo_root: &Path,
    profile: bool,
    format: FixtureFormat,
) -> Result<(ScriptResult, Option<ResourceStats>)> {
    let (mut command, working_dir) = match language {
        Language::Php => {
            let extension = ensure_php_extension(repo_root)?;
            let mut cmd = Command::new("php");
            cmd.arg("-d")
                .arg(format!("extension={}", extension.display()))
                .arg(repo_root.join("packages/php/bin/benchmark.php"));
            (cmd, repo_root.to_path_buf())
        }
        Language::Ruby => {
            let ruby_dir = ruby_runtime_dir();
            let mut cmd = Command::new(ruby_executable(&ruby_dir));
            cmd.arg(repo_root.join("packages/ruby/bin/benchmark.rb"));
            if let Some(dir) = ruby_dir {
                let mut paths: Vec<PathBuf> = Vec::new();
                paths.push(dir.clone());
                if let Ok(existing) = env::var("PATH") {
                    paths.extend(env::split_paths(&existing));
                }
                let joined = env::join_paths(paths)?;
                cmd.env("PATH", joined);

                let ruby_path = dir.join("ruby");
                cmd.env("RB_SYS_RUBY", &ruby_path);
                cmd.env("RUBY", &ruby_path);
            }
            cmd.env(
                "RUBYLIB",
                repo_root.join("packages/ruby/lib").to_string_lossy().to_string(),
            );
            (cmd, repo_root.join("packages/ruby"))
        }
        Language::Python => {
            let mut cmd = Command::new("uv");
            cmd.arg("run").arg("python").arg("bin/benchmark.py");
            (cmd, repo_root.join("packages/python"))
        }
        Language::Node => {
            ensure_node_binding(repo_root)?;
            let mut cmd = Command::new("pnpm");
            cmd.arg("--filter")
                .arg("html-to-markdown-node")
                .arg("exec")
                .arg("tsx")
                .arg("bin/benchmark.ts");
            (cmd, repo_root.to_path_buf())
        }
        Language::Wasm => {
            let mut cmd = Command::new("pnpm");
            cmd.arg("--filter")
                .arg("html-to-markdown-wasm")
                .arg("exec")
                .arg("tsx")
                .arg("bin/benchmark.ts");
            (cmd, repo_root.to_path_buf())
        }
        Language::Rust => bail!("Rust benchmarking is handled internally"),
    };

    command
        .arg("--file")
        .arg(file)
        .arg("--iterations")
        .arg(iterations.to_string())
        .arg("--format")
        .arg(format.as_str())
        .current_dir(&working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());

    let child = command
        .spawn()
        .with_context(|| format!("Failed to run {:?}", command))?;

    let monitor = if profile {
        ResourceMonitor::start(child.id(), Duration::from_millis(15))
    } else {
        None
    };

    let output = child.wait_with_output()?;
    let resource_stats = monitor.map(|m| m.stop());

    if !output.status.success() {
        bail!("{} benchmark script exited with status {}", language, output.status);
    }

    let stdout = String::from_utf8(output.stdout)?;
    let script_result: ScriptResult = serde_json::from_str(stdout.trim())?;
    Ok((script_result, resource_stats))
}

fn run_rust_benchmark(
    file: &Path,
    html: &str,
    iterations: u32,
    format: FixtureFormat,
    flamegraph_path: Option<&Path>,
) -> Result<(ScriptResult, Option<ResourceStats>)> {
    use html_to_markdown_rs::{ConversionOptions, convert};

    let mut options = ConversionOptions::default();
    if matches!(format, FixtureFormat::Hocr) {
        options.hocr_spatial_tables = false;
    }

    convert(html, Some(options.clone())).context("Rust warmup conversion failed")?;

    #[cfg(target_os = "windows")]
    if flamegraph_path.is_some() {
        bail!("--flamegraph isn't supported on Windows hosts (pprof depends on Unix tooling)");
    }

    #[cfg(not(target_os = "windows"))]
    let mut guard = if let Some(path) = flamegraph_path {
        Some((
            ProfilerGuardBuilder::default().frequency(100).build()?,
            path.to_path_buf(),
        ))
    } else {
        None
    };

    let start = Instant::now();
    for _ in 0..iterations {
        convert(html, Some(options.clone())).context("Rust conversion failed during benchmark")?;
    }
    let elapsed = start.elapsed().as_secs_f64();

    #[cfg(not(target_os = "windows"))]
    {
        if let Some((profile_guard, path)) = guard.take()
            && let Ok(report) = profile_guard.report().build()
        {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create flamegraph directory {}", parent.display()))?;
            }
            if let Err(err) = report.flamegraph(fs::File::create(&path)?) {
                eprintln!("Failed to write flamegraph: {err}");
            } else {
                println!("Flamegraph saved to {}", path.display());
            }
        }
    }

    let bytes_processed = html.len() * iterations as usize;
    let ops_per_sec = iterations as f64 / elapsed;
    let mb_per_sec = (bytes_processed as f64 / (1024.0 * 1024.0)) / elapsed;

    let script_result = ScriptResult {
        language: "rust".to_string(),
        fixture: file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string(),
        fixture_path: Some(file.to_path_buf()),
        iterations,
        elapsed_seconds: elapsed,
        ops_per_sec,
        mb_per_sec,
        bytes_processed,
    };

    Ok((script_result, None))
}

fn print_summary(results: &[HarnessResult]) {
    println!(
        "| Fixture                 | Lang   | Fmt  |  Ops/sec |    MB/s | Payload Size | Iter | Peak Mem | Avg CPU |"
    );
    println!(
        "| ----------------------- | ------ | ---- | -------- | ------- | ------------ | ---- | -------- | ------- |"
    );
    for result in results {
        println!("{}", result.table_row());
    }

    let mut best_by_language: BTreeMap<Language, &HarnessResult> = BTreeMap::new();
    for result in results {
        best_by_language
            .entry(result.language)
            .and_modify(|current| {
                if result.ops_per_sec > current.ops_per_sec {
                    *current = result;
                }
            })
            .or_insert(result);
    }

    if !best_by_language.is_empty() {
        println!("\nFastest fixtures per language:");
        for (language, result) in best_by_language {
            println!(
                "- {language}: {ops:.0} ops/sec on {fixture}",
                ops = result.ops_per_sec,
                fixture = result.fixture_name
            );
        }
    }
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

    println!("Building PHP extension (cargo build -p html-to-markdown-php --release)...");
    let status = Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("html-to-markdown-php")
        .arg("--release")
        .current_dir(repo_root)
        .status()?;

    if !status.success() {
        bail!("Failed to build html-to-markdown-php (status: {status})");
    }

    if candidate.exists() {
        Ok(candidate)
    } else {
        bail!("PHP extension not found at {} even after building", candidate.display());
    }
}

fn ensure_node_binding(repo_root: &Path) -> Result<()> {
    let node_dir = repo_root.join("crates/html-to-markdown-node");
    if has_node_binding(&node_dir) {
        return Ok(());
    }

    println!("Building Node.js native binding (pnpm --filter html-to-markdown-node run build)...");
    let status = Command::new("pnpm")
        .arg("--filter")
        .arg("html-to-markdown-node")
        .arg("run")
        .arg("build")
        .current_dir(repo_root)
        .status()?;

    if status.success() && has_node_binding(&node_dir) {
        Ok(())
    } else {
        bail!("Node.js binding build failed with status {status}");
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

fn ruby_executable(ruby_dir: &Option<PathBuf>) -> PathBuf {
    if let Some(dir) = ruby_dir {
        let candidate = dir.join("ruby");
        if candidate.exists() {
            return candidate;
        }
    }
    PathBuf::from("ruby")
}

fn ruby_runtime_dir() -> Option<PathBuf> {
    if let Ok(dir) = env::var("RUBY_BIN") {
        let path = PathBuf::from(&dir);
        if path.exists() {
            return Some(path);
        }
    }
    let default = PathBuf::from("/opt/homebrew/opt/ruby/bin");
    if default.exists() {
        return Some(default);
    }
    None
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
enum FixtureFormat {
    #[default]
    Html,
    Hocr,
}

impl FixtureFormat {
    fn as_str(&self) -> &'static str {
        match self {
            FixtureFormat::Html => "html",
            FixtureFormat::Hocr => "hocr",
        }
    }
}
