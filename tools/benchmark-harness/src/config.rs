use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BenchmarkMode {
    SingleFile,
    Batch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BenchmarkScenario {
    ConvertDefault,
    ConvertWithOptions,
    InlineImagesDefault,
    InlineImagesWithOptions,
    MetadataDefault,
    MetadataWithOptions,
}

impl BenchmarkScenario {
    pub fn all() -> Vec<Self> {
        vec![
            Self::ConvertDefault,
            Self::ConvertWithOptions,
            Self::InlineImagesDefault,
            Self::InlineImagesWithOptions,
            Self::MetadataDefault,
            Self::MetadataWithOptions,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ConvertDefault => "convert-default",
            Self::ConvertWithOptions => "convert-options",
            Self::InlineImagesDefault => "inline-images-default",
            Self::InlineImagesWithOptions => "inline-images-options",
            Self::MetadataDefault => "metadata-default",
            Self::MetadataWithOptions => "metadata-options",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    pub fixtures_path: PathBuf,
    pub output_dir: PathBuf,
    pub benchmark_mode: BenchmarkMode,
    pub warmup_iterations: usize,
    pub benchmark_iterations: usize,
    pub scenarios: Vec<BenchmarkScenario>,
    pub sample_interval_ms: u64,
    pub timeout: Duration,
    pub enable_profiling: bool,
    pub profile_frequency: i32,
    pub profile_repeat: usize,
    pub flamegraph_dir: Option<PathBuf>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            fixtures_path: PathBuf::from("test_documents"),
            output_dir: PathBuf::from("tools/benchmark-harness/results"),
            benchmark_mode: BenchmarkMode::SingleFile,
            warmup_iterations: 2,
            benchmark_iterations: 5,
            scenarios: BenchmarkScenario::all(),
            sample_interval_ms: 10,
            timeout: Duration::from_secs(300),
            enable_profiling: false,
            profile_frequency: 1000,
            profile_repeat: 1,
            flamegraph_dir: None,
        }
    }
}

impl BenchmarkConfig {
    pub fn validate(&self) -> crate::Result<()> {
        if self.benchmark_iterations == 0 {
            return Err(crate::Error::Config("benchmark_iterations must be > 0".to_string()));
        }
        if self.scenarios.is_empty() {
            return Err(crate::Error::Config("scenarios must not be empty".to_string()));
        }
        if self.sample_interval_ms == 0 {
            return Err(crate::Error::Config("sample_interval_ms must be > 0".to_string()));
        }
        if self.timeout.as_secs() == 0 {
            return Err(crate::Error::Config("timeout must be > 0".to_string()));
        }
        if self.profile_frequency <= 0 {
            return Err(crate::Error::Config("profile_frequency must be > 0".to_string()));
        }
        if self.profile_repeat == 0 {
            return Err(crate::Error::Config("profile_repeat must be > 0".to_string()));
        }
        Ok(())
    }
}
