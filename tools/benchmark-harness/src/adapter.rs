use crate::Result;
use crate::config::BenchmarkConfig;
use crate::fixture::{Fixture, FixtureFormat};
use crate::types::BenchmarkResult;

pub trait FrameworkAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn supports_format(&self, format: FixtureFormat) -> bool;
    fn run(&self, fixture: &Fixture, config: &BenchmarkConfig) -> Result<BenchmarkResult>;
}
