use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};

/// Application configuration with comprehensive seeding and debug options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Download directory
    pub download_dir: PathBuf,
    
    /// Upload rate limit in KB/s (0 = unlimited)
    pub upload_rate_limit: u32,
    
    /// Download rate limit in KB/s (0 = unlimited)
    pub download_rate_limit: u32,
    
    /// Maximum concurrent connections
    pub max_connections: u32,
    
    /// Listen port for torrent connections
    pub listen_port: u16,
    
    /// Web server port for remote UI
    pub web_ui_port: u16,
    
    /// Enable logging to file
    pub enable_file_logging: bool,
    
    /// Log directory
    pub log_dir: PathBuf,
    
    /// Prioritize seeding torrents with fewer seeders (0-100, 0=disabled)
    pub seed_prioritization: u8,
    
    /// Maximum torrents to seed simultaneously
    pub max_seeding_torrents: u32,
    
    /// Enable folder scanning on startup
    pub auto_scan_folders: bool,
    
    /// Folders to scan for complete torrents
    pub scan_folders: Vec<PathBuf>,
    
    /// Minimum seeders threshold for seeding
    pub min_seeders_threshold: u32,
    
    /// Add verbose console debug output
    pub verbose_logging: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            download_dir: PathBuf::from("./downloads"),
            upload_rate_limit: 0,
            download_rate_limit: 0,
            max_connections: 100,
            listen_port: 6881,
            web_ui_port: 8080,
            enable_file_logging: true,
            log_dir: PathBuf::from("./logs"),
            seed_prioritization: 80, // Enabled by default
            max_seeding_torrents: 10,
            auto_scan_folders: false,
            scan_folders: vec![],
            min_seeders_threshold: 1,
            verbose_logging: true,
        }
    }
}

impl Config {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Load configuration from TOML file, fallback to defaults if file missing
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        match fs::read_to_string(path) {
            Ok(content) => {
                let config: Config = toml::from_str(&content)
                    .context("Failed to parse TOML config file")?;
                Ok(config)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                tracing::info!("Config file not found, using defaults: {:?}", path);
                Ok(Self::default())
            }
            Err(e) => Err(anyhow::anyhow!("Failed to read config file: {}", e)),
        }
    }

    /// Save configuration to TOML file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let config_string = toml::to_string_pretty(self)
            .context("Failed to serialize config to TOML")?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(path, config_string)
            .context("Failed to write config file")?;
        
        tracing::info!("Config saved to: {:?}", path);
        Ok(())
    }

    /// Get default config file path
    pub fn default_config_path() -> PathBuf {
        #[cfg(target_os = "linux")]
        {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("bittorrent-client")
                .join("config.toml")
        }
        #[cfg(target_os = "windows")]
        {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("bittorrent-client")
                .join("config.toml")
        }
        #[cfg(target_os = "macos")]
        {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("bittorrent-client")
                .join("config.toml")
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        {
            PathBuf::from("config.toml")
        }
    }

    /// Ensure necessary directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.download_dir)?;
        fs::create_dir_all(&self.log_dir)?;
        Ok(())
    }
}

