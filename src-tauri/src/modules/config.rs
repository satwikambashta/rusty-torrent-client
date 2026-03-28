use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub download_dir: PathBuf,
    pub upload_rate_limit: u32, // KB/s
    pub download_rate_limit: u32, // KB/s
    pub max_connections: u32,
    pub listen_port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_dir: PathBuf::from("./downloads"),
            upload_rate_limit: 0, // No limit
            download_rate_limit: 0, // No limit
            max_connections: 100,
            listen_port: 6881,
        }
    }
}

impl Config {
    /// Create a new configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from file
    pub fn load_from_file(_path: &PathBuf) -> anyhow::Result<Self> {
        // TODO: Implement file loading
        Ok(Self::default())
    }

    /// Save configuration to file
    pub fn save_to_file(&self, _path: &PathBuf) -> anyhow::Result<()> {
        // TODO: Implement file saving
        Ok(())
    }
}
