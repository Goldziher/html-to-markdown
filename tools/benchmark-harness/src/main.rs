#[cfg(feature = "memory-profiling")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use benchmark_harness::adapters::{NativeAdapter, ScriptAdapter, ScriptLanguage};
use benchmark_harness::fixture::load_fixtures;
use benchmark_harness::{AdapterRegistry, BenchmarkConfig, BenchmarkMode, BenchmarkRunner, Result};
use clap::{Parser, Subcommand, ValueEnum};
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

        #[arg(long)]
        flamegraphs: Option<PathBuf>,

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

impl From<CliMode> for BenchmarkMode {
    fn from(mode: CliMode) -> Self {
        match mode {
            CliMode::SingleFile => BenchmarkMode::SingleFile,
            CliMode::Batch => BenchmarkMode::Batch,
        }
    }
}

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
        Commands::Run {
            fixtures: fixtures_path,
            frameworks,
            output,
            mode,
            warmup,
            iterations,
            profile,
            flamegraphs,
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

            let config = BenchmarkConfig {
                fixtures_path: fixtures_path.clone(),
                output_dir: output.clone(),
                benchmark_mode: mode.into(),
                warmup_iterations: warmup,
                benchmark_iterations: iterations,
                enable_profiling: profile || flamegraph_dir.is_some(),
                flamegraph_dir,
                ..Default::default()
            };
            config.validate()?;

            let mut registry = AdapterRegistry::new();
            registry.register(std::sync::Arc::new(NativeAdapter::new(repo_root.clone())))?;
            registry.register(std::sync::Arc::new(ScriptAdapter::new(
                ScriptLanguage::Python,
                repo_root.clone(),
            )))?;
            registry.register(std::sync::Arc::new(ScriptAdapter::new(
                ScriptLanguage::Ruby,
                repo_root.clone(),
            )))?;
            registry.register(std::sync::Arc::new(ScriptAdapter::new(
                ScriptLanguage::Php,
                repo_root.clone(),
            )))?;
            registry.register(std::sync::Arc::new(ScriptAdapter::new(
                ScriptLanguage::Node,
                repo_root.clone(),
            )))?;
            registry.register(std::sync::Arc::new(ScriptAdapter::new(
                ScriptLanguage::Wasm,
                repo_root.clone(),
            )))?;
            registry.register(std::sync::Arc::new(ScriptAdapter::new(
                ScriptLanguage::Java,
                repo_root.clone(),
            )))?;
            registry.register(std::sync::Arc::new(ScriptAdapter::new(
                ScriptLanguage::CSharp,
                repo_root.clone(),
            )))?;
            registry.register(std::sync::Arc::new(ScriptAdapter::new(
                ScriptLanguage::Go,
                repo_root.clone(),
            )))?;
            registry.register(std::sync::Arc::new(ScriptAdapter::new(
                ScriptLanguage::Elixir,
                repo_root.clone(),
            )))?;

            let runner = BenchmarkRunner::new(config, registry);
            let results = runner.run(&fixtures, &frameworks)?;

            let json_path = output.join("results.json");
            let html_path = output.join("report.html");

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
