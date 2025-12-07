use std::{fs, io, path::Path};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub listen_addr: String,
    pub target_addr: String,
    pub metrics_addr: String,
    pub grace_period_secs: u64,
    pub metrics_log_interval_secs: u64,
    pub channel_buffer_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:0".to_string(),
            target_addr: "127.0.0.1:0".to_string(),
            metrics_addr: "127.0.0.1:0".to_string(),
            grace_period_secs: 60,
            metrics_log_interval_secs: 10,
            channel_buffer_size: 1000,
        }
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
