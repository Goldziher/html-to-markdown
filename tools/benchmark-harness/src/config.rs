use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchmarkMode {
    SingleFile,
    Batch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub fixtures_path: PathBuf,
    pub output_dir: PathBuf,
    pub benchmark_mode: BenchmarkMode,
    pub warmup_iterations: usize,
    pub benchmark_iterations: usize,
    pub sample_interval_ms: u64,
    pub timeout: Duration,
    pub enable_profiling: bool,
    pub flamegraph_dir: Option<PathBuf>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            fixtures_path: PathBuf::from("tools/benchmark-harness/fixtures/wikipedia.toml"),
            output_dir: PathBuf::from("tools/benchmark-harness/results"),
            benchmark_mode: BenchmarkMode::SingleFile,
            warmup_iterations: 1,
            benchmark_iterations: 3,
            sample_interval_ms: 10,
            timeout: Duration::from_secs(300),
            enable_profiling: false,
            flamegraph_dir: None,
        }
    }
}

impl BenchmarkConfig {
    pub fn validate(&self) -> crate::Result<()> {
        if self.benchmark_iterations == 0 {
            return Err(crate::Error::Config("benchmark_iterations must be > 0".to_string()));
        }
        if self.sample_interval_ms == 0 {
            return Err(crate::Error::Config("sample_interval_ms must be > 0".to_string()));
        }
        if self.timeout.as_secs() == 0 {
            return Err(crate::Error::Config("timeout must be > 0".to_string()));
        }
        Ok(())
    }
}
