/// Configuration & Settings Tauri IPC Commands
///
/// Provides frontend access to configuration management.
/// All commands serialize/deserialize through JSON for IPC.

use crate::modules::config_manager::{AppConfig, ConfigManager};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

/// Configuration state wrapper for Tauri
pub struct ConfigState(pub ConfigManager);

/// Response for configuration operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub message: String,
    pub config: Option<AppConfig>,
}

/// Get current configuration
#[tauri::command]
pub async fn get_config(state: State<'_, ConfigState>) -> Result<AppConfig, String> {
    Ok(state.0.get().await)
}

/// Update entire configuration
#[tauri::command]
pub async fn update_config(
    config: AppConfig,
    state: State<'_, ConfigState>,
) -> Result<ConfigResponse, String> {
    match state.0.update(config.clone()).await {
        Ok(_) => {
            let _ = state.0.save().await;
            Ok(ConfigResponse {
                success: true,
                message: "Configuration updated successfully".to_string(),
                config: Some(config),
            })
        }
        Err(e) => Ok(ConfigResponse {
            success: false,
            message: format!("Failed to update configuration: {}", e),
            config: None,
        }),
    }
}

/// Update download directory
#[tauri::command]
pub async fn set_download_directory(
    directory: PathBuf,
    state: State<'_, ConfigState>,
) -> Result<ConfigResponse, String> {
    match state.0.set_download_dir(directory.clone()).await {
        Ok(_) => {
            let _ = state.0.save().await;
            let config = state.0.get().await;
            Ok(ConfigResponse {
                success: true,
                message: format!(
                    "Download directory updated to: {}",
                    directory.display()
                ),
                config: Some(config),
            })
        }
        Err(e) => Ok(ConfigResponse {
            success: false,
            message: format!("Failed to set download directory: {}", e),
            config: None,
        }),
    }
}

/// Update upload and download rate limits
#[tauri::command]
pub async fn set_rate_limits(
    upload_rate: u32,
    download_rate: u32,
    state: State<'_, ConfigState>,
) -> Result<ConfigResponse, String> {
    match state.0.set_rate_limits(upload_rate, download_rate).await {
        Ok(_) => {
            let _ = state.0.save().await;
            let config = state.0.get().await;
            Ok(ConfigResponse {
                success: true,
                message: format!(
                    "Rate limits updated: upload={}B/s, download={}B/s",
                    upload_rate, download_rate
                ),
                config: Some(config),
            })
        }
        Err(e) => Ok(ConfigResponse {
            success: false,
            message: format!("Failed to set rate limits: {}", e),
            config: None,
        }),
    }
}

/// Update peer connection limits
#[tauri::command]
pub async fn set_peer_limits(
    max_peers: usize,
    max_uploading: usize,
    state: State<'_, ConfigState>,
) -> Result<ConfigResponse, String> {
    match state.0.set_peer_limits(max_peers, max_uploading).await {
        Ok(_) => {
            let _ = state.0.save().await;
            let config = state.0.get().await;
            Ok(ConfigResponse {
                success: true,
                message: format!(
                    "Peer limits updated: max_peers={}, max_uploading={}",
                    max_peers, max_uploading
                ),
                config: Some(config),
            })
        }
        Err(e) => Ok(ConfigResponse {
            success: false,
            message: format!("Failed to set peer limits: {}", e),
            config: None,
        }),
    }
}

/// Toggle DHT
#[tauri::command]
pub async fn set_dht_enabled(
    enabled: bool,
    state: State<'_, ConfigState>,
) -> Result<ConfigResponse, String> {
    match state.0.set_dht_enabled(enabled).await {
        Ok(_) => {
            let _ = state.0.save().await;
            let config = state.0.get().await;
            Ok(ConfigResponse {
                success: true,
                message: format!("DHT {}", if enabled { "enabled" } else { "disabled" }),
                config: Some(config),
            })
        }
        Err(e) => Ok(ConfigResponse {
            success: false,
            message: format!("Failed to toggle DHT: {}", e),
            config: None,
        }),
    }
}

/// Toggle tracker
#[tauri::command]
pub async fn set_tracker_enabled(
    enabled: bool,
    state: State<'_, ConfigState>,
) -> Result<ConfigResponse, String> {
    match state.0.set_tracker_enabled(enabled).await {
        Ok(_) => {
            let _ = state.0.save().await;
            let config = state.0.get().await;
            Ok(ConfigResponse {
                success: true,
                message: format!("Tracker {}", if enabled { "enabled" } else { "disabled" }),
                config: Some(config),
            })
        }
        Err(e) => Ok(ConfigResponse {
            success: false,
            message: format!("Failed to toggle tracker: {}", e),
            config: None,
        }),
    }
}

/// Toggle verbose logging
#[tauri::command]
pub async fn set_verbose_logging(
    enabled: bool,
    state: State<'_, ConfigState>,
) -> Result<ConfigResponse, String> {
    match state.0.set_verbose_logging(enabled).await {
        Ok(_) => {
            let _ = state.0.save().await;
            let config = state.0.get().await;
            Ok(ConfigResponse {
                success: true,
                message: format!(
                    "Verbose logging {}",
                    if enabled { "enabled" } else { "disabled" }
                ),
                config: Some(config),
            })
        }
        Err(e) => Ok(ConfigResponse {
            success: false,
            message: format!("Failed to set verbose logging: {}", e),
            config: None,
        }),
    }
}

/// Update seeding limits
#[tauri::command]
pub async fn set_seeding_limits(
    seed_ratio_limit: f64,
    seed_idle_minutes: u64,
    state: State<'_, ConfigState>,
) -> Result<ConfigResponse, String> {
    match state.0.set_seeding_limits(seed_ratio_limit, seed_idle_minutes).await {
        Ok(_) => {
            let _ = state.0.save().await;
            let config = state.0.get().await;
            Ok(ConfigResponse {
                success: true,
                message: format!(
                    "Seeding limits updated: ratio={}, idle_minutes={}",
                    seed_ratio_limit, seed_idle_minutes
                ),
                config: Some(config),
            })
        }
        Err(e) => Ok(ConfigResponse {
            success: false,
            message: format!("Failed to set seeding limits: {}", e),
            config: None,
        }),
    }
}

/// Reset configuration to defaults
#[tauri::command]
pub async fn reset_config_to_defaults(
    state: State<'_, ConfigState>,
) -> Result<ConfigResponse, String> {
    match state.0.reset_to_defaults().await {
        Ok(_) => {
            let _ = state.0.save().await;
            let config = state.0.get().await;
            Ok(ConfigResponse {
                success: true,
                message: "Configuration reset to defaults".to_string(),
                config: Some(config),
            })
        }
        Err(e) => Ok(ConfigResponse {
            success: false,
            message: format!("Failed to reset configuration: {}", e),
            config: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_response_serialization() {
        let response = ConfigResponse {
            success: true,
            message: "Test".to_string(),
            config: Some(AppConfig::default()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ConfigResponse = serde_json::from_str(&json).unwrap();
        assert!(deserialized.success);
    }
}