use std::{fs, path::Path};

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
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn select_panels<'a>(&'a self, names: &[String]) -> Result<Vec<&'a Panel>, Vec<String>> {
        if names.is_empty() {
            return Ok(self.panels.iter().collect());
        }

        let unknown: Vec<String> = names
            .iter()
            .filter(|n| !self.panels.iter().any(|p| p.name.eq_ignore_ascii_case(n)))
            .cloned()
            .collect();

        if !unknown.is_empty() {
            return Err(unknown);
        }

        Ok(self
            .panels
            .iter()
            .filter(|p| names.iter().any(|n| n.eq_ignore_ascii_case(&p.name)))
            .collect())
    }
}
