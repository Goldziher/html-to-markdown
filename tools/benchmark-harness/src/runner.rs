use crate::adapter::FrameworkAdapter;
use crate::config::BenchmarkConfig;
use crate::fixture::Fixture;
use crate::registry::AdapterRegistry;
use crate::types::{BenchmarkResult, PerformanceMetrics, ResourceStats};
use crate::{Error, Result};
use std::sync::Arc;
use std::time::Duration;

pub struct BenchmarkRunner {
    config: BenchmarkConfig,
    registry: AdapterRegistry,
}

impl BenchmarkRunner {
    pub fn new(config: BenchmarkConfig, registry: AdapterRegistry) -> Self {
        Self { config, registry }
    }

    pub fn run(&self, fixtures: &[Fixture], frameworks: &[String]) -> Result<Vec<BenchmarkResult>> {
        let adapters = if frameworks.is_empty() {
            self.registry.adapters()
        } else {
            frameworks
                .iter()
                .map(|name| {
                    self.registry
                        .get(name)
                        .ok_or_else(|| Error::UnsupportedFramework(name.clone()))
                })
                .collect::<Result<Vec<_>>>()?
        };

        let mut results = Vec::new();

        for fixture in fixtures {
            for adapter in &adapters {
                if !adapter.supports_format(fixture.format) {
                    continue;
                }

                let result = run_adapter(adapter.clone(), fixture, &self.config)
                    .unwrap_or_else(|err| failed_result(adapter.clone(), fixture, err));
                results.push(result);
            }
        }

        Ok(results)
    }
}

fn run_adapter(
    adapter: Arc<dyn FrameworkAdapter>,
    fixture: &Fixture,
    config: &BenchmarkConfig,
) -> Result<BenchmarkResult> {
    adapter.run(fixture, config)
}

fn failed_result(adapter: Arc<dyn FrameworkAdapter>, fixture: &Fixture, err: Error) -> BenchmarkResult {
    BenchmarkResult {
        framework: adapter.name().to_string(),
        fixture_id: fixture.id.clone(),
        fixture_name: fixture.name.clone(),
        fixture_path: fixture.path.clone(),
        fixture_format: fixture.format.as_str().to_string(),
        file_extension: fixture.file_extension(),
        file_size: 0,
        iterations: 0,
        duration: Duration::from_secs(0),
        metrics: PerformanceMetrics::default(),
        resource_stats: ResourceStats::default(),
        memory_stats: None,
        flamegraph_path: None,
        statistics: None,
        success: false,
        error_message: Some(err.to_string()),
    }
}
