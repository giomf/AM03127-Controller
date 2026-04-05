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
}
