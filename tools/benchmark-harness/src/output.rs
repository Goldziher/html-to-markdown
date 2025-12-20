use crate::types::BenchmarkResult;
use crate::{Error, Result};
use minijinja::{Environment, context};
use std::fs;
use std::path::Path;

const REPORT_TEMPLATE: &str = include_str!("../templates/report.html.jinja");

pub fn write_json_results(results: &[BenchmarkResult], output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(Error::Io)?;
    }

    let json = serde_json::to_string_pretty(results)
        .map_err(|err| Error::Serialization(format!("Failed to serialize results: {err}")))?;
    fs::write(output_path, json).map_err(Error::Io)?;
    Ok(())
}

pub fn write_html_report(results: &[BenchmarkResult], output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(Error::Io)?;
    }

    let mut env = Environment::new();
    env.add_template("report", REPORT_TEMPLATE)
        .map_err(|err| Error::Serialization(format!("Failed to load template: {err}")))?;

    let summary = build_summary(results);
    let rows = results.iter().map(ReportRow::from).collect::<Vec<_>>();

    let rendered = env
        .get_template("report")
        .map_err(|err| Error::Serialization(format!("Failed to get template: {err}")))?
        .render(context! { results => rows, summary => summary })
        .map_err(|err| Error::Serialization(format!("Failed to render template: {err}")))?;

    fs::write(output_path, rendered).map_err(Error::Io)?;
    Ok(())
}

fn build_summary(results: &[BenchmarkResult]) -> SummaryStats {
    let total = results.len();
    let successes = results.iter().filter(|r| r.success).count();
    let failures = total.saturating_sub(successes);

    let avg_ops = average(results.iter().map(|r| r.metrics.ops_per_sec));
    let avg_mb = average(results.iter().map(|r| r.metrics.mb_per_sec));

    SummaryStats {
        total,
        successes,
        failures,
        avg_ops,
        avg_mb,
    }
}

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

#[derive(serde::Serialize)]
struct SummaryStats {
    total: usize,
    successes: usize,
    failures: usize,
    avg_ops: f64,
    avg_mb: f64,
}

#[derive(serde::Serialize)]
struct ReportRow {
    framework: String,
    fixture_name: String,
    fixture_format: String,
    ops_per_sec: f64,
    mb_per_sec: f64,
    duration_ms: f64,
    peak_memory_mb: f64,
    avg_cpu_percent: f64,
    flamegraph_path: Option<String>,
}

impl From<&BenchmarkResult> for ReportRow {
    fn from(result: &BenchmarkResult) -> Self {
        Self {
            framework: result.framework.clone(),
            fixture_name: result.fixture_name.clone(),
            fixture_format: result.fixture_format.clone(),
            ops_per_sec: result.metrics.ops_per_sec,
            mb_per_sec: result.metrics.mb_per_sec,
            duration_ms: result.duration.as_secs_f64() * 1000.0,
            peak_memory_mb: result.resource_stats.peak_memory_bytes as f64 / 1_048_576.0,
            avg_cpu_percent: result.resource_stats.avg_cpu_percent,
            flamegraph_path: result
                .flamegraph_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
        }
    }
}
