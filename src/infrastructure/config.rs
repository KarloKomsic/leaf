// Simple TOML-based config that stores the library directory path.
// Lives in XDG_CONFIG_HOME/leaf/config.toml so it persists across
// sessions without any manual setup.

use crate::infrastructure::paths::config_file_path;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub library_directory: Option<String>,
}

#[derive(Debug)]
pub enum ConfigError {
    Io,
    Parse,
}

impl Config {
    pub fn new() -> Self {
        Self {
            library_directory: None,
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let toml_string = toml::to_string(self).expect("Failed to serialize config");

        let path = config_file_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        fs::write(path, toml_string)
    }

    pub fn load() -> Result<Self, ConfigError> {
        let path = config_file_path();
        let content = std::fs::read_to_string(path).map_err(|_| ConfigError::Io)?;

        let config = toml::from_str(&content).map_err(|_| ConfigError::Parse)?;

        Ok(config)
    }
}
