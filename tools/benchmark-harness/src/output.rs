use crate::types::BenchmarkResult;
use crate::{Error, Result};
use minijinja::{Environment, context};
use std::collections::HashMap;
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
    let framework_summary = build_framework_summary(results);
    let fixture_summary = build_fixture_summary(results);
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
        overall: build_summary(results),
        frameworks: build_framework_summary(results),
        fixtures: build_fixture_summary(results),
    };

    let json = serde_json::to_string_pretty(&summary)
        .map_err(|err| Error::Serialization(format!("Failed to serialize summary: {err}")))?;
    fs::write(output_path, json).map_err(Error::Io)?;
    Ok(())
}

fn build_summary(results: &[BenchmarkResult]) -> SummaryStats {
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

fn build_framework_summary(results: &[BenchmarkResult]) -> Vec<FrameworkSummary> {
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

fn build_fixture_summary(results: &[BenchmarkResult]) -> Vec<FixtureSummary> {
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

fn median(mut values: Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    values[values.len() / 2]
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
struct FrameworkSummary {
    framework: String,
    runs: usize,
    successes: usize,
    median_ops: f64,
    median_mb: f64,
    peak_memory_mb: f64,
    avg_cpu_percent: f64,
}

#[derive(serde::Serialize)]
struct FixtureSummary {
    fixture: String,
    format: String,
    scenario: String,
    runs: usize,
    successes: usize,
    median_ops: f64,
    median_mb: f64,
    peak_memory_mb: f64,
}

#[derive(serde::Serialize)]
struct SummaryReport {
    overall: SummaryStats,
    frameworks: Vec<FrameworkSummary>,
    fixtures: Vec<FixtureSummary>,
}

#[derive(serde::Serialize)]
struct Hotspot {
    name: String,
    samples: usize,
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

fn extract_flamegraph_hotspots(path: &Path, limit: usize) -> Result<Vec<Hotspot>> {
    let data = fs::read_to_string(path).map_err(Error::Io)?;
    if let Some(hotspots) = extract_0x_hotspots(&data, limit) {
        return Ok(hotspots);
    }
    let mut totals: HashMap<String, usize> = HashMap::new();

    for chunk in data.split("<title>").skip(1) {
        let Some(end) = chunk.find("</title>") else { continue };
        let title = &chunk[..end];
        let (name, meta) = match title.rsplit_once(" (") {
            Some((name, meta)) => (name, meta.trim_end_matches(')')),
            None => continue,
        };
        let samples = parse_samples(meta);
        if samples == 0 {
            continue;
        }
        if should_ignore_frame(name) {
            continue;
        }
        *totals.entry(name.to_string()).or_insert(0) += samples;
    }

    let mut entries = totals
        .into_iter()
        .map(|(name, samples)| Hotspot { name, samples })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.samples.cmp(&a.samples));
    entries.truncate(limit);
    Ok(entries)
}

fn extract_0x_hotspots(data: &str, limit: usize) -> Option<Vec<Hotspot>> {
    let start = data.find("visualizer(")? + "visualizer(".len();
    let tail = &data[start..];
    let end = tail.find(");")?;
    let payload = tail[..end].trim();
    if payload.is_empty() {
        return None;
    }

    let mut totals: HashMap<String, usize> = HashMap::new();
    let mut index = 0;
    while let Some(pos) = payload[index..].find("\"name\":\"") {
        let name_start = index + pos + "\"name\":\"".len();
        let mut name = String::new();
        let mut escaped = false;
        let mut cursor = name_start;
        for (offset, ch) in payload[name_start..].char_indices() {
            cursor = name_start + offset;
            if escaped {
                name.push(ch);
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == '"' {
                cursor += 1;
                break;
            }
            name.push(ch);
        }

        if name.is_empty() || should_ignore_frame(&name) {
            index = cursor;
            continue;
        }

        if let Some(value_pos) = payload[cursor..].find("\"value\":") {
            let mut value_index = cursor + value_pos + "\"value\":".len();
            while let Some(ch) = payload[value_index..].chars().next() {
                if ch.is_whitespace() {
                    value_index += ch.len_utf8();
                } else {
                    break;
                }
            }
            let mut value_str = String::new();
            for ch in payload[value_index..].chars() {
                if ch.is_ascii_digit() || ch == '.' {
                    value_str.push(ch);
                } else {
                    break;
                }
            }
            if let Ok(value) = value_str.parse::<f64>() {
                if value > 0.0 {
                    *totals.entry(name).or_insert(0) += value as usize;
                }
            }
        }

        index = cursor;
    }

    let mut entries = totals
        .into_iter()
        .map(|(name, samples)| Hotspot { name, samples })
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| b.samples.cmp(&a.samples));
    entries.truncate(limit);
    Some(entries)
}

fn parse_samples(meta: &str) -> usize {
    let mut digits = String::new();
    for ch in meta.chars() {
        if ch.is_ascii_digit() || ch == ',' {
            digits.push(ch);
        } else {
            break;
        }
    }
    digits.replace(',', "").parse::<usize>().unwrap_or(0)
}

fn should_ignore_frame(name: &str) -> bool {
    matches!(name, "all" | "all stacks" | "__libc_start_main" | "_start")
        || name.starts_with("benchmark-harne")
        || name.starts_with("benchmark-harness")
}
