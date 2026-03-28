use serde::{Deserialize, Serialize};

/// DTO for torrent info returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub id: String,
    pub name: String,
    pub total_size: u64,
    pub downloaded: u64,
    pub uploaded: u64,
    pub status: String,
    pub progress: f32,
}

/// Tauri command to fetch all active torrents
#[tauri::command]
pub fn get_torrents() -> Vec<TorrentInfo> {
    // TODO: Fetch from TorrentManager
    vec![]
}

/// Tauri command to add a new torrent
#[tauri::command]
pub fn add_torrent(_torrent_path: String) -> Result<String, String> {
    // TODO: Integrate with TorrentManager
    Ok("torrent_id_123".to_string())
}

/// Tauri command to start a torrent
#[tauri::command]
pub fn start_torrent(_torrent_id: String) -> Result<(), String> {
    // TODO: Integrate with TorrentManager
    Ok(())
}

/// Tauri command to pause a torrent
#[tauri::command]
pub fn pause_torrent(_torrent_id: String) -> Result<(), String> {
    // TODO: Integrate with TorrentManager
    Ok(())
}

/// Tauri command to remove a torrent
#[tauri::command]
pub fn remove_torrent(_torrent_id: String) -> Result<(), String> {
    // TODO: Integrate with TorrentManager
    Ok(())
}
