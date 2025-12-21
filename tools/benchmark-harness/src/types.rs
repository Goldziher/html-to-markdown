use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub framework: String,
    #[serde(default = "default_scenario")]
    pub scenario: String,
    pub fixture_id: String,
    pub fixture_name: String,
    pub fixture_path: PathBuf,
    pub fixture_format: String,
    pub file_extension: String,
    pub file_size: u64,
    pub iterations: usize,
    pub duration: Duration,
    pub metrics: PerformanceMetrics,
    pub resource_stats: ResourceStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory_stats: Option<MemoryStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flamegraph_path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<DurationStatistics>,
    #[serde(default)]
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

fn default_scenario() -> String {
    "convert-default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PerformanceMetrics {
    pub ops_per_sec: f64,
    pub mb_per_sec: f64,
    pub throughput_bytes_per_sec: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceStats {
    pub peak_memory_bytes: u64,
    pub avg_cpu_percent: f64,
    pub p50_memory_bytes: u64,
    pub p95_memory_bytes: u64,
    pub p99_memory_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub allocated_bytes: u64,
    pub resident_bytes: u64,
    pub active_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationResult {
    pub iteration: usize,
    pub duration: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DurationStatistics {
    pub mean: Duration,
    pub median: Duration,
    pub std_dev_ms: f64,
    pub min: Duration,
    pub max: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub sample_count: usize,
}
