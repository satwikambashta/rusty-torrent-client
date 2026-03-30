/// Configuration & Settings Module
///
/// Manages:
/// - Application settings (download dir, rate limits, etc.)
/// - Settings persistence (JSON file-based)
/// - Configuration validation
/// - Default configurations
/// - Settings change notifications

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::fs;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Download directory path
    pub download_dir: PathBuf,
    /// Maximum concurrent downloads
    pub max_concurrent_downloads: usize,
    /// Maximum upload rate (bytes/sec), 0 = unlimited
    pub max_upload_rate: u32,
    /// Maximum download rate (bytes/sec), 0 = unlimited
    pub max_download_rate: u32,
    /// Maximum uploading peers per torrent
    pub max_uploading_peers: usize,
    /// Maximum peer connections
    pub max_peer_connections: usize,
    /// Enable DHT
    pub enable_dht: bool,
    /// Enable tracker announces
    pub enable_tracker: bool,
    /// Enable PEX (Peer Exchange)
    pub enable_pex: bool,
    /// Enable UPnP port mapping
    pub enable_upnp: bool,
    /// Listen port for incoming connections
    pub listen_port: u16,
    /// Verbose logging
    pub verbose_logging: bool,
    /// Auto-start downloads on startup
    pub auto_start_downloads: bool,
    /// Stop seeding when ratio reaches (0.0 = unlimited)
    pub seed_ratio_limit: f64,
    /// Stop seeding when idle for X minutes (0 = unlimited)
    pub seed_idle_limit_minutes: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let download_dir = home.join("Downloads").join("Torrents");

        Self {
            download_dir,
            max_concurrent_downloads: 5,
            max_upload_rate: 0, // Unlimited
            max_download_rate: 0, // Unlimited
            max_uploading_peers: 4,
            max_peer_connections: 200,
            enable_dht: true,
            enable_tracker: true,
            enable_pex: false,
            enable_upnp: false,
            listen_port: 6881,
            verbose_logging: false,
            auto_start_downloads: false,
            seed_ratio_limit: 0.0,
            seed_idle_limit_minutes: 0,
        }
    }
}

impl AppConfig {
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check download directory
        if self.download_dir.as_os_str().is_empty() {
            return Err("Download directory cannot be empty".to_string());
        }

        // Check limits
        if self.max_concurrent_downloads == 0 {
            return Err("Max concurrent downloads must be > 0".to_string());
        }

        if self.max_uploading_peers == 0 {
            return Err("Max uploading peers must be > 0".to_string());
        }

        if self.max_peer_connections == 0 {
            return Err("Max peer connections must be > 0".to_string());
        }

        if self.listen_port == 0 {
            return Err("Listen port must be > 0".to_string());
        }

        if self.seed_ratio_limit < 0.0 {
            return Err("Seed ratio limit cannot be negative".to_string());
        }

        Ok(())
    }

    /// Create download directory if it doesn't exist
    pub fn ensure_download_dir(&self) -> Result<(), std::io::Error> {
        fs::create_dir_all(&self.download_dir)
    }

    /// Clone with updated download directory
    pub fn with_download_dir(&self, dir: PathBuf) -> Self {
        let mut config = self.clone();
        config.download_dir = dir;
        config
    }

    /// Clone with updated rate limits
    pub fn with_rate_limits(&self, upload: u32, download: u32) -> Self {
        let mut config = self.clone();
        config.max_upload_rate = upload;
        config.max_download_rate = download;
        config
    }

    /// Clone with updated peer limits
    pub fn with_peer_limits(&self, max_peers: usize, max_uploading: usize) -> Self {
        let mut config = self.clone();
        config.max_peer_connections = max_peers;
        config.max_uploading_peers = max_uploading;
        config
    }

    /// Clone with updated seeding limits
    pub fn with_seeding_limits(&self, ratio: f64, idle_minutes: u64) -> Self {
        let mut config = self.clone();
        config.seed_ratio_limit = ratio;
        config.seed_idle_limit_minutes = idle_minutes;
        config
    }
}

/// Configuration manager
pub struct ConfigManager {
    config: Arc<Mutex<AppConfig>>,
    config_file: PathBuf,
}

impl ConfigManager {
    /// Create new config manager with default config
    pub fn new(config_file: PathBuf) -> Self {
        Self {
            config: Arc::new(Mutex::new(AppConfig::default())),
            config_file,
        }
    }

    /// Load configuration from file
    pub async fn load(&mut self) -> Result<(), String> {
        if !self.config_file.exists() {
            // Create default config file
            self.save().await?;
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.config_file)
            .await
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        config.validate()?;

        let mut current = self.config.lock().await;
        *current = config;

        Ok(())
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<(), String> {
        let config = self.config.lock().await;
        let json = serde_json::to_string_pretty(&*config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        // Ensure config directory exists
        if let Some(parent) = self.config_file.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        tokio::fs::write(&self.config_file, json)
            .await
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Get current configuration
    pub async fn get(&self) -> AppConfig {
        self.config.lock().await.clone()
    }

    /// Update configuration
    pub async fn update(&self, config: AppConfig) -> Result<(), String> {
        config.validate()?;
        config
            .ensure_download_dir()
            .map_err(|e| format!("Failed to ensure download directory: {}", e))?;

        let mut current = self.config.lock().await;
        *current = config;

        Ok(())
    }

    /// Update download directory
    pub async fn set_download_dir(&self, dir: PathBuf) -> Result<(), String> {
        let mut config = self.config.lock().await;
        config.download_dir = dir.clone();
        drop(config);

        let config = self.config.lock().await;
        config
            .ensure_download_dir()
            .map_err(|e| format!("Failed to ensure download directory: {}", e))?;
        Ok(())
    }

    /// Update rate limits
    pub async fn set_rate_limits(&self, upload: u32, download: u32) -> Result<(), String> {
        let mut config = self.config.lock().await;
        config.max_upload_rate = upload;
        config.max_download_rate = download;
        Ok(())
    }

    /// Update peer connection limits
    pub async fn set_peer_limits(&self, max_peers: usize, max_uploading: usize) -> Result<(), String> {
        if max_peers == 0 || max_uploading == 0 {
            return Err("Peer limits must be > 0".to_string());
        }

        let mut config = self.config.lock().await;
        config.max_peer_connections = max_peers;
        config.max_uploading_peers = max_uploading;
        Ok(())
    }

    /// Toggle DHT
    pub async fn set_dht_enabled(&self, enabled: bool) -> Result<(), String> {
        let mut config = self.config.lock().await;
        config.enable_dht = enabled;
        Ok(())
    }

    /// Toggle tracker
    pub async fn set_tracker_enabled(&self, enabled: bool) -> Result<(), String> {
        let mut config = self.config.lock().await;
        config.enable_tracker = enabled;
        Ok(())
    }

    /// Toggle verbose logging
    pub async fn set_verbose_logging(&self, enabled: bool) -> Result<(), String> {
        let mut config = self.config.lock().await;
        config.verbose_logging = enabled;
        Ok(())
    }

    /// Set seed limits
    pub async fn set_seeding_limits(&self, ratio: f64, idle_minutes: u64) -> Result<(), String> {
        if ratio < 0.0 {
            return Err("Seed ratio cannot be negative".to_string());
        }

        let mut config = self.config.lock().await;
        config.seed_ratio_limit = ratio;
        config.seed_idle_limit_minutes = idle_minutes;
        Ok(())
    }

    /// Reset to default configuration
    pub async fn reset_to_defaults(&self) -> Result<(), String> {
        let default_config = AppConfig::default();
        default_config
            .ensure_download_dir()
            .map_err(|e| format!("Failed to ensure download directory: {}", e))?;

        let mut config = self.config.lock().await;
        *config = default_config;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.download_dir.ends_with("Torrents"));
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::default();
        assert!(config.validate().is_ok());

        config.max_concurrent_downloads = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_with_download_dir() {
        let config = AppConfig::default();
        let new_config = config.with_download_dir(PathBuf::from("/tmp/torrents"));
        assert_eq!(new_config.download_dir, PathBuf::from("/tmp/torrents"));
        assert_eq!(config.download_dir, AppConfig::default().download_dir);
    }

    #[test]
    fn test_config_with_rate_limits() {
        let config = AppConfig::default();
        let new_config = config.with_rate_limits(1024, 2048);
        assert_eq!(new_config.max_upload_rate, 1024);
        assert_eq!(new_config.max_download_rate, 2048);
    }

    #[test]
    fn test_config_with_peer_limits() {
        let config = AppConfig::default();
        let new_config = config.with_peer_limits(300, 8);
        assert_eq!(new_config.max_peer_connections, 300);
        assert_eq!(new_config.max_uploading_peers, 8);
    }

    #[test]
    fn test_config_with_seeding_limits() {
        let config = AppConfig::default();
        let new_config = config.with_seeding_limits(2.0, 60);
        assert_eq!(new_config.seed_ratio_limit, 2.0);
        assert_eq!(new_config.seed_idle_limit_minutes, 60);
    }

    #[tokio::test]
    async fn test_config_manager_creation() {
        let manager = ConfigManager::new(PathBuf::from("/tmp/test_config.json"));
        let config = manager.get().await;
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_config_manager_set_download_dir() {
        let manager = ConfigManager::new(PathBuf::from("/tmp/test_config.json"));
        let result = manager.set_download_dir(PathBuf::from("/tmp/torrents")).await;
        assert!(result.is_ok());

        let config = manager.get().await;
        assert_eq!(config.download_dir, PathBuf::from("/tmp/torrents"));
    }

    #[tokio::test]
    async fn test_config_manager_set_rate_limits() {
        let manager = ConfigManager::new(PathBuf::from("/tmp/test_config.json"));
        let result = manager.set_rate_limits(1024, 2048).await;
        assert!(result.is_ok());

        let config = manager.get().await;
        assert_eq!(config.max_upload_rate, 1024);
        assert_eq!(config.max_download_rate, 2048);
    }

    #[tokio::test]
    async fn test_config_manager_set_peer_limits() {
        let manager = ConfigManager::new(PathBuf::from("/tmp/test_config.json"));
        let result = manager.set_peer_limits(300, 8).await;
        assert!(result.is_ok());

        let config = manager.get().await;
        assert_eq!(config.max_peer_connections, 300);
        assert_eq!(config.max_uploading_peers, 8);
    }

    #[tokio::test]
    async fn test_config_manager_reset() {
        let manager = ConfigManager::new(PathBuf::from("/tmp/test_config.json"));
        manager.set_download_dir(PathBuf::from("/tmp/custom")).await.unwrap();

        let result = manager.reset_to_defaults().await;
        assert!(result.is_ok());

        let config = manager.get().await;
        assert_eq!(config.max_upload_rate, 0); // Reset to default
    }
}