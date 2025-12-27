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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum VisitorType {
    #[default]
    None,
    Noop,
    Simple,
    Custom,
    Complex,
}

impl VisitorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VisitorType::None => "none",
            VisitorType::Noop => "noop",
            VisitorType::Simple => "simple",
            VisitorType::Custom => "custom",
            VisitorType::Complex => "complex",
        }
    }
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
    #[serde(default)]
    pub visitor: VisitorType,
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
    let mut toml_files = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(Error::Io)? {
        let entry = entry.map_err(Error::Io)?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("toml") {
            toml_files.push(path);
        }
    }

    if !toml_files.is_empty() {
        let mut fixtures = Vec::new();
        for path in toml_files {
            fixtures.extend(load_fixtures_from_file(&path)?);
        }
        return Ok(fixtures);
    }

    load_fixtures_from_documents(dir)
}

fn load_fixtures_from_documents(dir: &Path) -> Result<Vec<Fixture>> {
    let cwd = std::env::current_dir().map_err(Error::Io)?;
    let mut fixtures = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(path) = stack.pop() {
        for entry in std::fs::read_dir(&path).map_err(Error::Io)? {
            let entry = entry.map_err(Error::Io)?;
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
                continue;
            }

            let extension = entry_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            let format = match extension.as_str() {
                "html" | "htm" => FixtureFormat::Html,
                "hocr" => FixtureFormat::Hocr,
                _ => continue,
            };

            let relative_path = entry_path
                .strip_prefix(&cwd)
                .unwrap_or(entry_path.as_path())
                .to_path_buf();

            let relative_to_dir = entry_path.strip_prefix(dir).unwrap_or(entry_path.as_path());
            let id = sanitize_id(relative_to_dir);
            let name = entry_path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or(id.as_str())
                .to_string();
            let category = relative_to_dir.parent().and_then(|parent| {
                if parent.as_os_str().is_empty() {
                    None
                } else {
                    Some(path_to_category(parent))
                }
            });

            let file_size = std::fs::metadata(&entry_path).map(|m| m.len()).unwrap_or(0);
            let iterations = Some(suggested_iterations(file_size));

            let fixture = Fixture {
                id,
                name,
                path: relative_path,
                category,
                iterations,
                format,
                visitor: VisitorType::default(),
            };

            fixture.validate(dir)?;
            fixtures.push(fixture);
        }
    }

    if fixtures.is_empty() {
        return Err(Error::Fixture(format!(
            "No fixture definitions found in {}",
            dir.display()
        )));
    }

    fixtures.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(fixtures)
}

fn suggested_iterations(file_size: u64) -> u32 {
    if file_size < 25_000 {
        15
    } else if file_size < 100_000 {
        10
    } else if file_size < 500_000 {
        7
    } else if file_size < 2_000_000 {
        5
    } else {
        3
    }
}

fn sanitize_id(path: &Path) -> String {
    let mut id = String::new();
    let mut prev_dash = false;

    for ch in path.to_string_lossy().chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '-'
        };
        if mapped == '-' {
            if prev_dash {
                continue;
            }
            prev_dash = true;
        } else {
            prev_dash = false;
        }
        id.push(mapped);
    }

    id.trim_matches('-').to_string()
}

fn path_to_category(path: &Path) -> String {
    let mut category = path.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
    category.make_ascii_lowercase();
    category
}
