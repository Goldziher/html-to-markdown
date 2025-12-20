use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Fixture error: {0}")]
    Fixture(String),
    #[error("Benchmark error: {0}")]
    Benchmark(String),
    #[error("Invalid fixture file: {path} ({reason})")]
    InvalidFixture { path: PathBuf, reason: String },
    #[error("Unsupported framework: {0}")]
    UnsupportedFramework(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Profiling error: {0}")]
    Profiling(String),
}

pub type Result<T> = std::result::Result<T, Error>;
