pub mod adapter;
pub mod adapters;
pub mod config;
pub mod error;
pub mod fixture;
pub mod monitoring;
pub mod output;
pub mod profiling;
pub mod registry;
pub mod runner;
pub mod types;

pub use crate::config::{BenchmarkConfig, BenchmarkMode};
pub use crate::error::{Error, Result};
pub use crate::fixture::{Fixture, FixtureFormat, FixtureSet};
pub use crate::output::{write_html_report, write_json_results, write_summary_json};
pub use crate::registry::AdapterRegistry;
pub use crate::runner::BenchmarkRunner;
