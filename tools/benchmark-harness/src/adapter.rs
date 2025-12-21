use crate::Result;
use crate::config::BenchmarkConfig;
use crate::config::BenchmarkScenario;
use crate::fixture::{Fixture, FixtureFormat};
use crate::types::BenchmarkResult;

pub trait FrameworkAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn supports_format(&self, format: FixtureFormat) -> bool;
    fn supports_scenario(&self, scenario: BenchmarkScenario) -> bool {
        let _ = scenario;
        true
    }
    fn run(&self, fixture: &Fixture, scenario: BenchmarkScenario, config: &BenchmarkConfig) -> Result<BenchmarkResult>;
}
