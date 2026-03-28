use serde::{Deserialize, Serialize};

/// DTO for search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultDto {
    pub name: String,
    pub size: u64,
    pub seeders: u32,
    pub leechers: u32,
    pub magnet: String,
}

/// DTO for scanned files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFileDto {
    pub path: String,
    pub size: u64,
    pub md5: String,
    pub sha1: String,
}

/// Search for torrents
#[tauri::command]
pub async fn search_torrents(query: String, limit: usize) -> Result<Vec<SearchResultDto>, String> {
    use crate::modules::search::TorrentSearchService;

    match TorrentSearchService::search(&query, limit).await {
        Ok(results) => Ok(results
            .into_iter()
            .map(|r| SearchResultDto {
                name: r.name,
                size: r.size,
                seeders: r.seeders,
                leechers: r.leechers,
                magnet: r.magnet,
            })
            .collect()),
        Err(e) => Err(format!("Search failed: {}", e)),
    }
}

/// Scan a folder for complete files
#[tauri::command]
pub fn scan_folder(path: String) -> Result<Vec<ScannedFileDto>, String> {
    use crate::modules::scanner::FolderScanner;
    use std::path::Path;

    match FolderScanner::scan_folder(Path::new(&path), None) {
        Ok(files) => Ok(files
            .into_iter()
            .map(|f| ScannedFileDto {
                path: f.path.to_string_lossy().to_string(),
                size: f.size,
                md5: f.md5,
                sha1: f.sha1,
            })
            .collect()),
        Err(e) => Err(format!("Scan failed: {}", e)),
    }
}

/// Get configuration
#[tauri::command]
pub fn get_config() -> Result<serde_json::Value, String> {
    use crate::modules::config::Config;

    let config_path = Config::default_config_path();
    match Config::load_from_file(&config_path) {
        Ok(config) => {
            match serde_json::to_value(config) {
                Ok(value) => Ok(value),
                Err(e) => Err(format!("Failed to serialize config: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to load config: {}", e)),
    }
}

/// Update configuration
#[tauri::command]
pub fn update_config(config_json: serde_json::Value) -> Result<(), String> {
    use crate::modules::config::Config;

    match serde_json::from_value::<Config>(config_json) {
        Ok(config) => {
            let config_path = Config::default_config_path();
            match config.save_to_file(&config_path) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to save config: {}", e)),
            }
        }
        Err(e) => Err(format!("Invalid config: {}", e)),
    }
}

/// Get seeding statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct SeedingStats {
    pub active_torrents: usize,
    pub total_uploaded: u64,
    pub avg_seeders: f32,
    pub optimization_score: f32,
}

#[tauri::command]
pub fn get_seeding_stats() -> Result<SeedingStats, String> {
    // TODO: Integrate with actual torrent manager
    Ok(SeedingStats {
        active_torrents: 0,
        total_uploaded: 0,
        avg_seeders: 0.0,
        optimization_score: 0.0,
    })
}
