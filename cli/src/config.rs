use std::{fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Panel {
    pub name: String,
    pub address: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub panels: Vec<Panel>,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read '{}'", path.display()))?;
        toml::from_str(&contents).context("failed to parse config")
    }

    pub fn select_panels<'a>(&'a self, names: &[String]) -> Result<Vec<&'a Panel>> {
        if names.is_empty() {
            return Ok(self.panels.iter().collect());
        }

        let unknown: Vec<&str> = names
            .iter()
            .filter(|n| !self.panels.iter().any(|p| p.name.eq_ignore_ascii_case(n)))
            .map(String::as_str)
            .collect();

        if !unknown.is_empty() {
            bail!("unknown panel(s): {}", unknown.join(", "));
        }

        Ok(self
            .panels
            .iter()
            .filter(|p| names.iter().any(|n| n.eq_ignore_ascii_case(&p.name)))
            .collect())
    }
}
