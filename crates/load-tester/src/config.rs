use std::{fs, io, path::Path};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default = "default_connections")]
    pub connections: usize,
    #[serde(default = "default_duration")]
    pub duration_secs: u64,
    #[serde(default = "default_message_size")]
    pub message_size: usize,
}

fn default_connections() -> usize {
    10
}
fn default_duration() -> u64 {
    10
}
fn default_message_size() -> usize {
    1024
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub target_addr: String,
    pub connections: usize,
    pub duration_secs: u64,
    pub message_size: usize,
    #[serde(default)]
    pub scenarios: Vec<Scenario>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target_addr: "127.0.0.1:8080".to_string(),
            connections: 10,
            duration_secs: 30,
            message_size: 1024,
            scenarios: Vec::new(),
        }
    }
}

impl Config {
    pub fn is_matrix_mode(&self) -> bool {
        !self.scenarios.is_empty()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }
}
