//! Summary statistics generation for benchmark reports.

use crate::types::BenchmarkResult;
use std::collections::HashMap;

/// Overall benchmark summary statistics.
#[derive(serde::Serialize)]
pub struct SummaryStats {
    pub total: usize,
    pub successes: usize,
    pub failures: usize,
    pub avg_ops: f64,
    pub avg_mb: f64,
}

/// Per-framework summary statistics.
#[derive(serde::Serialize)]
pub struct FrameworkSummary {
    pub framework: String,
    pub runs: usize,
    pub successes: usize,
    pub median_ops: f64,
    pub median_mb: f64,
    pub peak_memory_mb: f64,
    pub avg_cpu_percent: f64,
}

/// Per-fixture summary statistics.
#[derive(serde::Serialize)]
pub struct FixtureSummary {
    pub fixture: String,
    pub format: String,
    pub scenario: String,
    pub runs: usize,
    pub successes: usize,
    pub median_ops: f64,
    pub median_mb: f64,
    pub peak_memory_mb: f64,
}

/// Complete summary report with all sections.
#[derive(serde::Serialize)]
pub struct SummaryReport {
    pub overall: SummaryStats,
    pub frameworks: Vec<FrameworkSummary>,
    pub fixtures: Vec<FixtureSummary>,
}

/// Build overall benchmark summary.
pub fn build_summary(results: &[BenchmarkResult]) -> SummaryStats {
    let total = results.len();
    let successes = results.iter().filter(|r| r.success).count();
    let failures = total.saturating_sub(successes);

    let avg_ops = average(results.iter().filter(|r| r.success).map(|r| r.metrics.ops_per_sec));
    let avg_mb = average(results.iter().filter(|r| r.success).map(|r| r.metrics.mb_per_sec));

    SummaryStats {
        total,
        successes,
        failures,
        avg_ops,
        avg_mb,
    }
}

/// Build per-framework summary statistics.
pub fn build_framework_summary(results: &[BenchmarkResult]) -> Vec<FrameworkSummary> {
    let mut by_framework: HashMap<&str, Vec<&BenchmarkResult>> = HashMap::new();
    for result in results {
        by_framework.entry(&result.framework).or_default().push(result);
    }

    let mut summaries = by_framework
        .into_iter()
        .map(|(framework, entries)| {
            let successes = entries.iter().filter(|r| r.success).collect::<Vec<_>>();
            let median_ops = median(successes.iter().map(|r| r.metrics.ops_per_sec).collect());
            let median_mb = median(successes.iter().map(|r| r.metrics.mb_per_sec).collect());
            let peak_memory_mb = successes
                .iter()
                .map(|r| r.resource_stats.peak_memory_bytes as f64 / 1_048_576.0)
                .fold(0.0, f64::max);
            let avg_cpu_percent = average(successes.iter().map(|r| r.resource_stats.avg_cpu_percent));

            FrameworkSummary {
                framework: framework.to_string(),
                runs: entries.len(),
                successes: successes.len(),
                median_ops,
                median_mb,
                peak_memory_mb,
                avg_cpu_percent,
            }
        })
        .collect::<Vec<_>>();

    summaries.sort_by(|a, b| a.framework.cmp(&b.framework));
    summaries
}

/// Build per-fixture summary statistics.
pub fn build_fixture_summary(results: &[BenchmarkResult]) -> Vec<FixtureSummary> {
    let mut by_fixture: HashMap<(&str, &str, &str), Vec<&BenchmarkResult>> = HashMap::new();
    for result in results {
        by_fixture
            .entry((&result.fixture_name, &result.fixture_format, &result.scenario))
            .or_default()
            .push(result);
    }

    let mut summaries = by_fixture
        .into_iter()
        .map(|((fixture, format, scenario), entries)| {
            let successes = entries.iter().filter(|r| r.success).collect::<Vec<_>>();
            let median_ops = median(successes.iter().map(|r| r.metrics.ops_per_sec).collect());
            let median_mb = median(successes.iter().map(|r| r.metrics.mb_per_sec).collect());
            let peak_memory_mb = successes
                .iter()
                .map(|r| r.resource_stats.peak_memory_bytes as f64 / 1_048_576.0)
                .fold(0.0, f64::max);

            FixtureSummary {
                fixture: fixture.to_string(),
                format: format.to_string(),
                scenario: scenario.to_string(),
                runs: entries.len(),
                successes: successes.len(),
                median_ops,
                median_mb,
                peak_memory_mb,
            }
        })
        .collect::<Vec<_>>();

    summaries.sort_by(|a, b| match a.fixture.cmp(&b.fixture) {
        std::cmp::Ordering::Equal => a.scenario.cmp(&b.scenario),
        other => other,
    });
    summaries
}

/// Calculate average from an iterator of f64 values.
fn average<I>(values: I) -> f64
where
    I: Iterator<Item = f64>,
{
    let mut total = 0.0;
    let mut count = 0.0;
    for value in values {
        total += value;
        count += 1.0;
    }

    if count == 0.0 { 0.0 } else { total / count }
}

/// Calculate median from a vector of f64 values.
fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values[values.len() / 2]
}
