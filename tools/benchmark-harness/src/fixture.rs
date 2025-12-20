use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FixtureFormat {
    #[default]
    Html,
    Hocr,
}

impl FixtureFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            FixtureFormat::Html => "html",
            FixtureFormat::Hocr => "hocr",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureSet {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    pub fixtures: Vec<Fixture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub iterations: Option<u32>,
    #[serde(default)]
    pub format: FixtureFormat,
}

impl Fixture {
    pub fn validate(&self, fixture_file: &Path) -> Result<()> {
        if self.id.trim().is_empty() {
            return Err(Error::InvalidFixture {
                path: fixture_file.to_path_buf(),
                reason: "fixture id is empty".to_string(),
            });
        }
        if self.name.trim().is_empty() {
            return Err(Error::InvalidFixture {
                path: fixture_file.to_path_buf(),
                reason: "fixture name is empty".to_string(),
            });
        }
        if self.path.is_absolute() {
            return Err(Error::InvalidFixture {
                path: fixture_file.to_path_buf(),
                reason: "fixture path must be relative".to_string(),
            });
        }
        Ok(())
    }

    pub fn resolved_path(&self, repo_root: &Path) -> PathBuf {
        repo_root.join(&self.path)
    }

    pub fn file_extension(&self) -> String {
        self.path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("unknown")
            .to_lowercase()
    }
}

pub fn load_fixtures(path: &Path) -> Result<Vec<Fixture>> {
    if path.is_dir() {
        return load_fixtures_from_dir(path);
    }

    load_fixtures_from_file(path)
}

pub fn load_fixtures_from_file(path: &Path) -> Result<Vec<Fixture>> {
    let contents = std::fs::read_to_string(path).map_err(Error::Io)?;
    let set: FixtureSet = toml::from_str(&contents)
        .map_err(|err| Error::Fixture(format!("Failed to parse {}: {}", path.display(), err)))?;

    for fixture in &set.fixtures {
        fixture.validate(path)?;
    }

    Ok(set.fixtures)
}

pub fn load_fixtures_from_dir(dir: &Path) -> Result<Vec<Fixture>> {
    let mut fixtures = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(Error::Io)? {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            fixtures.extend(load_fixtures_from_file(&path)?);
        }
    }

    if fixtures.is_empty() {
        return Err(Error::Fixture(format!(
            "No fixture definitions found in {}",
            dir.display()
        )));
    }

    Ok(fixtures)
}
