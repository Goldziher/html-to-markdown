use crate::types::BenchmarkResult;
use crate::{Error, Result};
use minijinja::{Environment, context};
use std::fs;
use std::path::Path;

mod hotspots;
mod summaries;

use hotspots::{Hotspot, extract_flamegraph_hotspots};
use summaries::SummaryReport;

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

    let summary = summaries::build_summary(results);
    let framework_summary = summaries::build_framework_summary(results);
    let fixture_summary = summaries::build_fixture_summary(results);
    let output_dir = output_path.parent().unwrap_or_else(|| Path::new("."));
    let rows = results
        .iter()
        .map(|result| ReportRow::from_result(result, output_dir))
        .collect::<Vec<_>>();

    let rendered = env
        .get_template("report")
        .map_err(|err| Error::Serialization(format!("Failed to get template: {err}")))?
        .render(context! {
            results => rows,
            summary => summary,
            frameworks => framework_summary,
            fixtures => fixture_summary
        })
        .map_err(|err| Error::Serialization(format!("Failed to render template: {err}")))?;

    fs::write(output_path, rendered).map_err(Error::Io)?;
    Ok(())
}

pub fn write_summary_json(results: &[BenchmarkResult], output_path: &Path) -> Result<()> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(Error::Io)?;
    }

    let summary = SummaryReport {
        overall: summaries::build_summary(results),
        frameworks: summaries::build_framework_summary(results),
        fixtures: summaries::build_fixture_summary(results),
    };

    let json = serde_json::to_string_pretty(&summary)
        .map_err(|err| Error::Serialization(format!("Failed to serialize summary: {err}")))?;
    fs::write(output_path, json).map_err(Error::Io)?;
    Ok(())
}

#[derive(serde::Serialize)]
struct ReportRow {
    framework: String,
    scenario: String,
    fixture_name: String,
    fixture_format: String,
    ops_per_sec: f64,
    mb_per_sec: f64,
    duration_ms: f64,
    peak_memory_mb: f64,
    avg_cpu_percent: f64,
    flamegraph_path: Option<String>,
    hotspots: Vec<Hotspot>,
}

impl ReportRow {
    fn from_result(result: &BenchmarkResult, output_dir: &Path) -> Self {
        let hotspots = result
            .flamegraph_path
            .as_ref()
            .and_then(|path| {
                let resolved = if path.is_relative() {
                    output_dir.join(path)
                } else {
                    path.clone()
                };
                extract_flamegraph_hotspots(&resolved, 5).ok()
            })
            .unwrap_or_default();

        Self {
            framework: result.framework.clone(),
            scenario: result.scenario.clone(),
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
            hotspots,
        }
    }
}
