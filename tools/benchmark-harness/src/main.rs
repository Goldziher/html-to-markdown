//! Benchmark harness CLI for html-to-markdown.
//!
//! Provides commands for running benchmarks across different frameworks and languages,
//! validating fixtures, and generating reports with performance metrics and flamegraphs.

#[cfg(feature = "memory-profiling")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use benchmark_harness::adapters::{NativeAdapter, ScriptAdapter, ScriptLanguage};
use benchmark_harness::fixture::load_fixtures;
use benchmark_harness::types::BenchmarkResult;
use benchmark_harness::{AdapterRegistry, BenchmarkConfig, BenchmarkMode, BenchmarkRunner, BenchmarkScenario, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "benchmark-harness")]
#[command(about = "Benchmark harness for html-to-markdown", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List fixtures from a file or directory
    ListFixtures {
        #[arg(short, long)]
        fixtures: PathBuf,
    },

    /// Validate fixture definitions
    Validate {
        #[arg(short, long)]
        fixtures: PathBuf,
    },

    /// Generate an HTML gallery for flamegraphs
    GenerateFlamegraphIndex {
        #[arg(long)]
        flamegraphs: PathBuf,
        #[arg(long)]
        output: PathBuf,
    },

    /// Consolidate benchmark results from multiple result directories
    Consolidate {
        #[arg(short, long)]
        input: PathBuf,
        #[arg(short, long, default_value = "tools/benchmark-harness/results-consolidated")]
        output: PathBuf,
    },

    /// Run benchmarks
    Run {
        #[arg(short, long)]
        fixtures: PathBuf,

        #[arg(short = 'F', long, value_delimiter = ',')]
        frameworks: Vec<String>,

        #[arg(short, long, default_value = "tools/benchmark-harness/results")]
        output: PathBuf,

        #[arg(short = 'm', long, value_enum, default_value = "single-file")]
        mode: CliMode,

        #[arg(short = 'w', long, default_value = "1")]
        warmup: usize,

        #[arg(short = 'i', long, default_value = "3")]
        iterations: usize,

        #[arg(long)]
        profile: bool,

        #[arg(long, default_value = "1000")]
        profile_frequency: i32,

        #[arg(long, default_value = "1")]
        profile_repeat: usize,

        #[arg(long)]
        flamegraphs: Option<PathBuf>,

        #[arg(long)]
        rust_baseline: bool,

        #[arg(long, value_enum, value_delimiter = ',')]
        scenarios: Vec<CliScenario>,

        #[arg(long, value_enum, default_value = "json")]
        format: OutputFormat,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliMode {
    #[value(name = "single-file")]
    SingleFile,
    #[value(name = "batch")]
    Batch,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Json,
    Html,
    Both,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliScenario {
    #[value(name = "convert-default")]
    ConvertDefault,
    #[value(name = "convert-options")]
    ConvertWithOptions,
    #[value(name = "inline-images-default")]
    InlineImagesDefault,
    #[value(name = "inline-images-options")]
    InlineImagesWithOptions,
    #[value(name = "metadata-default")]
    MetadataDefault,
    #[value(name = "metadata-options")]
    MetadataWithOptions,
    #[value(name = "metadata-raw")]
    MetadataRaw,
}

impl From<CliMode> for BenchmarkMode {
    fn from(mode: CliMode) -> Self {
        match mode {
            CliMode::SingleFile => Self::SingleFile,
            CliMode::Batch => Self::Batch,
        }
    }
}

impl From<CliScenario> for BenchmarkScenario {
    fn from(scenario: CliScenario) -> Self {
        match scenario {
            CliScenario::ConvertDefault => Self::ConvertDefault,
            CliScenario::ConvertWithOptions => Self::ConvertWithOptions,
            CliScenario::InlineImagesDefault => Self::InlineImagesDefault,
            CliScenario::InlineImagesWithOptions => Self::InlineImagesWithOptions,
            CliScenario::MetadataDefault => Self::MetadataDefault,
            CliScenario::MetadataWithOptions => Self::MetadataWithOptions,
            CliScenario::MetadataRaw => Self::MetadataRaw,
        }
    }
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::ListFixtures { fixtures } => {
            let fixtures = load_fixtures(&fixtures)?;
            println!("Loaded {} fixture(s)", fixtures.len());
            for fixture in fixtures {
                println!(
                    "- {} ({}) -> {}",
                    fixture.id,
                    fixture.format.as_str(),
                    fixture.path.display()
                );
            }
            Ok(())
        }
        Commands::Validate { fixtures } => {
            let fixtures = load_fixtures(&fixtures)?;
            println!("✓ {} fixture(s) validated", fixtures.len());
            Ok(())
        }
        Commands::GenerateFlamegraphIndex { flamegraphs, output } => {
            generate_flamegraph_index(&flamegraphs, &output)?;
            println!("✓ Flamegraph index generated: {}", output.display());
            Ok(())
        }
        Commands::Consolidate { input, output } => {
            let results = collect_results(&input)?;
            if results.is_empty() {
                return Err(benchmark_harness::Error::Config(format!(
                    "No results.json files found under {}",
                    input.display()
                )));
            }

            let json_path = output.join("results.json");
            let html_path = output.join("report.html");
            let summary_path = output.join("summary.json");

            benchmark_harness::write_json_results(&results, &json_path)?;
            benchmark_harness::write_html_report(&results, &html_path)?;
            benchmark_harness::write_summary_json(&results, &summary_path)?;

            println!("✓ Consolidated {} result(s) into {}", results.len(), output.display());
            Ok(())
        }
        Commands::Run {
            fixtures: fixtures_path,
            frameworks,
            output,
            mode,
            warmup,
            iterations,
            profile,
            profile_frequency,
            profile_repeat,
            flamegraphs,
            rust_baseline,
            scenarios,
            format,
        } => {
            let repo_root = repo_root()?;
            let fixtures = load_fixtures(&fixtures_path)?;
            let flamegraph_dir = flamegraphs.or_else(|| {
                if profile {
                    Some(output.join("flamegraphs"))
                } else {
                    None
                }
            });

            let scenarios = if scenarios.is_empty() {
                BenchmarkScenario::all()
            } else {
                scenarios.into_iter().map(Into::into).collect()
            };

            let config = BenchmarkConfig {
                fixtures_path,
                output_dir: output.clone(),
                benchmark_mode: mode.into(),
                warmup_iterations: warmup,
                benchmark_iterations: iterations,
                scenarios,
                enable_profiling: profile || flamegraph_dir.is_some(),
                profile_frequency,
                profile_repeat,
                flamegraph_dir,
                include_rust_baseline: rust_baseline,
                ..Default::default()
            };
            config.validate()?;

            let mut registry = AdapterRegistry::new();
            registry.register(std::sync::Arc::new(NativeAdapter::new(repo_root.clone())))?;

            // Register all script language adapters
            for language in ScriptLanguage::all() {
                registry.register(std::sync::Arc::new(ScriptAdapter::new(*language, repo_root.clone())))?;
            }

            let runner = BenchmarkRunner::new(config, registry);
            let results = runner.run(&fixtures, &frameworks)?;

            let json_path = output.join("results.json");
            let html_path = output.join("report.html");
            let summary_path = output.join("summary.json");

            match format {
                OutputFormat::Json => {
                    benchmark_harness::write_json_results(&results, &json_path)?;
                }
                OutputFormat::Html => {
                    benchmark_harness::write_html_report(&results, &html_path)?;
                }
                OutputFormat::Both => {
                    benchmark_harness::write_json_results(&results, &json_path)?;
                    benchmark_harness::write_html_report(&results, &html_path)?;
                }
            }
            benchmark_harness::write_summary_json(&results, &summary_path)?;

            Ok(())
        }
    }
}

fn repo_root() -> Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .ok_or_else(|| benchmark_harness::Error::Config("Failed to resolve repo root".to_string()))?;
    Ok(repo_root.to_path_buf())
}

fn generate_flamegraph_index(flamegraph_dir: &Path, output: &Path) -> Result<()> {
    let mut flamegraphs = Vec::new();
    for entry in std::fs::read_dir(flamegraph_dir).map_err(benchmark_harness::Error::Io)? {
        let entry = entry.map_err(benchmark_harness::Error::Io)?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("svg") {
            flamegraphs.push(path.file_name().unwrap().to_string_lossy().to_string());
        }
    }

    flamegraphs.sort();

    let mut env = minijinja::Environment::new();
    env.add_template("index", include_str!("../templates/flamegraphs.html.jinja"))
        .map_err(|err| benchmark_harness::Error::Serialization(format!("Failed to load template: {err}")))?;

    let html = env
        .get_template("index")
        .map_err(|err| benchmark_harness::Error::Serialization(format!("Failed to get template: {err}")))?
        .render(minijinja::context! { flamegraphs => flamegraphs })
        .map_err(|err| benchmark_harness::Error::Serialization(format!("Failed to render template: {err}")))?;

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(benchmark_harness::Error::Io)?;
    }

    std::fs::write(output, html).map_err(benchmark_harness::Error::Io)?;
    Ok(())
}

fn collect_results(input: &Path) -> Result<Vec<BenchmarkResult>> {
    let mut files = Vec::new();
    if input.is_file() {
        files.push(input.to_path_buf());
    } else {
        collect_results_files(input, &mut files)?;
    }

    let mut results = Vec::new();
    for file in files {
        if file.file_name().and_then(|name| name.to_str()) != Some("results.json") {
            continue;
        }
        let raw = std::fs::read_to_string(&file).map_err(benchmark_harness::Error::Io)?;
        let mut entries: Vec<BenchmarkResult> = serde_json::from_str(&raw).map_err(|err| {
            benchmark_harness::Error::Serialization(format!("Failed to parse {}: {err}", file.display()))
        })?;

        let base_dir = file.parent().unwrap_or_else(|| Path::new("."));
        for entry in &mut entries {
            if let Some(path) = entry.flamegraph_path.clone() {
                let absolute = if path.is_absolute() {
                    path
                } else {
                    to_absolute_path(base_dir)?.join(path)
                };
                entry.flamegraph_path = Some(absolute);
            }
        }
        results.extend(entries);
    }

    Ok(results)
}

fn collect_results_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir).map_err(benchmark_harness::Error::Io)? {
        let entry = entry.map_err(benchmark_harness::Error::Io)?;
        let path = entry.path();
        if path.is_dir() {
            collect_results_files(&path, files)?;
        } else if path.file_name().and_then(|name| name.to_str()) == Some("results.json") {
            files.push(path);
        }
    }
    Ok(())
}

fn to_absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    Ok(env::current_dir().map_err(benchmark_harness::Error::Io)?.join(path))
}
