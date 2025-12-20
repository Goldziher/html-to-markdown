use crate::adapter::FrameworkAdapter;
use crate::config::BenchmarkConfig;
use crate::fixture::{Fixture, FixtureFormat};
use crate::monitoring::ResourceMonitor;
#[cfg(all(feature = "profiling", not(target_os = "windows")))]
use crate::profiling::ProfileGuard;
use crate::types::{BenchmarkResult, DurationStatistics, IterationResult, MemoryStats, PerformanceMetrics};
use crate::{Error, Result};
use html_to_markdown_rs::{ConversionOptions, convert};
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct NativeAdapter {
    repo_root: PathBuf,
}

impl NativeAdapter {
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    fn read_fixture(&self, fixture: &Fixture) -> Result<String> {
        let path = fixture.resolved_path(&self.repo_root);
        let data = std::fs::read(&path).map_err(Error::Io)?;
        String::from_utf8(data).map_err(|_| Error::Benchmark(format!("Fixture {} is not valid UTF-8", path.display())))
    }

    fn build_options(format: FixtureFormat) -> ConversionOptions {
        let mut options = ConversionOptions::default();
        if matches!(format, FixtureFormat::Hocr) {
            options.hocr_spatial_tables = false;
        }
        options
    }
}

impl FrameworkAdapter for NativeAdapter {
    fn name(&self) -> &str {
        "rust"
    }

    fn supports_format(&self, format: FixtureFormat) -> bool {
        matches!(format, FixtureFormat::Html | FixtureFormat::Hocr)
    }

    fn run(&self, fixture: &Fixture, config: &BenchmarkConfig) -> Result<BenchmarkResult> {
        let html = self.read_fixture(fixture)?;
        let options = Self::build_options(fixture.format);
        let iterations = fixture.iterations.unwrap_or(config.benchmark_iterations as u32).max(1) as usize;

        for _ in 0..config.warmup_iterations.max(1) {
            convert(&html, Some(options.clone()))
                .map_err(|err| Error::Benchmark(format!("Warmup conversion failed: {err}")))?;
        }

        #[cfg(all(feature = "profiling", not(target_os = "windows")))]
        let mut profiler = if config.enable_profiling {
            Some(ProfileGuard::new(1000)?)
        } else {
            None
        };

        let monitor = ResourceMonitor::start(std::process::id(), Duration::from_millis(config.sample_interval_ms));

        let mut iteration_results = Vec::with_capacity(iterations);
        let start = Instant::now();
        for iteration in 0..iterations {
            let iter_start = Instant::now();
            convert(&html, Some(options.clone()))
                .map_err(|err| Error::Benchmark(format!("Conversion failed: {err}")))?;
            iteration_results.push(IterationResult {
                iteration,
                duration: iter_start.elapsed(),
            });
        }
        let total_duration = start.elapsed();

        let resource_stats = monitor.map(|m| m.stop()).unwrap_or_default();

        let file_size = html.len() as u64;
        let bytes_processed = file_size as f64 * iterations as f64;
        let duration_secs = total_duration.as_secs_f64().max(0.000_001);
        let ops_per_sec = iterations as f64 / duration_secs;
        let mb_per_sec = (bytes_processed / (1024.0 * 1024.0)) / duration_secs;

        let statistics = if iteration_results.len() > 1 {
            Some(calculate_statistics(&iteration_results))
        } else {
            None
        };

        let flamegraph_path = if config.enable_profiling {
            config
                .flamegraph_dir
                .clone()
                .map(|dir| dir.join(format!("{}-{}.svg", self.name(), fixture.id)))
        } else {
            None
        };

        #[cfg(all(feature = "profiling", not(target_os = "windows")))]
        if let (Some(profile_guard), Some(path)) = (profiler.take(), flamegraph_path.as_ref()) {
            let report = profile_guard.finish()?;
            report.generate_flamegraph(path)?;
        }

        let memory_stats = capture_memory_stats().ok();

        Ok(BenchmarkResult {
            framework: self.name().to_string(),
            fixture_id: fixture.id.clone(),
            fixture_name: fixture.name.clone(),
            fixture_path: fixture.resolved_path(&self.repo_root),
            fixture_format: fixture.format.as_str().to_string(),
            file_extension: fixture.file_extension(),
            file_size,
            iterations,
            duration: total_duration,
            metrics: PerformanceMetrics {
                ops_per_sec,
                mb_per_sec,
                throughput_bytes_per_sec: bytes_processed / duration_secs,
            },
            resource_stats,
            memory_stats,
            flamegraph_path,
            statistics,
            success: true,
            error_message: None,
        })
    }
}

fn calculate_statistics(iterations: &[IterationResult]) -> DurationStatistics {
    let mut durations: Vec<Duration> = iterations.iter().map(|r| r.duration).collect();
    durations.sort();

    let min = *durations.first().unwrap_or(&Duration::from_secs(0));
    let max = *durations.last().unwrap_or(&Duration::from_secs(0));

    let total_ms: f64 = durations.iter().map(|d| d.as_secs_f64() * 1000.0).sum();
    let mean_ms = total_ms / durations.len() as f64;
    let mean = Duration::from_secs_f64(mean_ms / 1000.0);

    let median = durations[durations.len() / 2];
    let p95 = durations[((durations.len() as f64 - 1.0) * 0.95) as usize];
    let p99 = durations[((durations.len() as f64 - 1.0) * 0.99) as usize];

    let variance: f64 = durations
        .iter()
        .map(|d| {
            let diff = d.as_secs_f64() * 1000.0 - mean_ms;
            diff * diff
        })
        .sum::<f64>()
        / durations.len() as f64;

    DurationStatistics {
        mean,
        median,
        std_dev_ms: variance.sqrt(),
        min,
        max,
        p95,
        p99,
        sample_count: durations.len(),
    }
}

fn capture_memory_stats() -> Result<MemoryStats> {
    #[cfg(feature = "memory-profiling")]
    {
        use tikv_jemalloc_ctl::{epoch, stats};
        epoch::advance().map_err(|err| Error::Profiling(format!("Failed to advance jemalloc epoch: {err}")))?;
        let allocated = stats::allocated::read()
            .map_err(|err| Error::Profiling(format!("Failed to read jemalloc allocated: {err}")))?;
        let resident = stats::resident::read()
            .map_err(|err| Error::Profiling(format!("Failed to read jemalloc resident: {err}")))?;
        let active =
            stats::active::read().map_err(|err| Error::Profiling(format!("Failed to read jemalloc active: {err}")))?;

        Ok(MemoryStats {
            allocated_bytes: allocated as u64,
            resident_bytes: resident as u64,
            active_bytes: active as u64,
        })
    }

    #[cfg(not(feature = "memory-profiling"))]
    {
        Err(Error::Profiling("Memory profiling is disabled".to_string()))
    }
}
