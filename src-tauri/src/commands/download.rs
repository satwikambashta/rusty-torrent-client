/// Download Command Handlers for Tauri IPC
///
/// Provides commands for:
/// - Loading torrent files
/// - Starting/pausing downloads
/// - Getting download progress
/// - Managing download sessions

use crate::modules::download::{DownloadEngine, DownloadEngineStats};
use crate::modules::pieces::PieceInfo;
use crate::modules::torrent_parser::TorrentParser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;

lazy_static::lazy_static! {
    static ref DOWNLOAD_ENGINES: Mutex<std::collections::HashMap<String, DownloadEngine>> = 
        Mutex::new(std::collections::HashMap::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgressPayload {
    pub session_id: String,
    pub stats: DownloadEngineStats,
}


/// DTO for torrent file info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTorrentInfo {
    pub info_hash: String,
    pub name: String,
    pub total_size: u64,
    pub piece_count: u32,
    pub announce: String,
}

/// Command to parse a torrent file
#[tauri::command]
pub fn parse_torrent_file(file_path: String) -> Result<ParsedTorrentInfo, String> {
    let path = PathBuf::from(&file_path);

    let metadata = TorrentParser::parse_file(&path)
        .map_err(|e| format!("Failed to parse torrent: {}", e))?;

    Ok(ParsedTorrentInfo {
        info_hash: metadata.info_hash_hex.clone(),
        name: metadata.name.clone(),
        total_size: metadata.total_length,
        piece_count: metadata.pieces_count,
        announce: metadata.announce.clone(),
    })
}

/// Command to start a download
#[tauri::command]
pub fn start_download(
    file_path: String,
    download_dir: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let path = PathBuf::from(&file_path);
    let dl_dir = PathBuf::from(&download_dir);

    // Parse torrent file
    let metadata = TorrentParser::parse_file(&path)
        .map_err(|e| format!("Failed to parse torrent: {}", e))?;

    let info_hash = metadata.info_hash_hex.clone();

    // Create download session
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let session = crate::modules::download::DownloadSession {
        id: uuid::Uuid::new_v4().to_string(),
        metadata: metadata.clone(),
        started_at: now,
        uploaded: 0,
    };

    // Create pieces from metadata
    let pieces = metadata
        .pieces
        .iter()
        .enumerate()
        .map(|(idx, hash)| {
            let piece_size = if (idx as u32) < metadata.pieces_count - 1 {
                metadata.piece_length
            } else {
                // Last piece might be smaller
                let last_piece_size = metadata.total_length % metadata.piece_length;
                if last_piece_size == 0 {
                    metadata.piece_length
                } else {
                    last_piece_size
                }
            };
            PieceInfo::new(idx as u32, hash.clone(), piece_size)
        })
        .collect::<Vec<_>>();

    // Create download engine
    let engine = DownloadEngine::new(session.clone(), dl_dir, pieces);

    // Store engine
    let session_id = session.id.clone();
    DOWNLOAD_ENGINES
        .lock()
        .unwrap()
        .insert(session_id.clone(), engine);

    tracing::info!(
        "Started download session {} for torrent {}",
        session_id,
        info_hash
    );

    // Spawn periodic progress event emitter
    let app_handle_clone = app_handle.clone();
    let session_id_clone = session_id.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;

            let maybe_stats = {
                let engines = DOWNLOAD_ENGINES.lock().unwrap();
                engines.get(&session_id_clone).map(|engine| engine.stats())
            };

            if let Some(stats) = maybe_stats {
                let payload = DownloadProgressPayload {
                    session_id: session_id_clone.clone(),
                    stats,
                };

                if let Err(err) = app_handle_clone.emit("download-progress", payload.clone()) {
                    tracing::warn!("Failed to emit download-progress event: {}", err);
                }
            } else {
                break;
            }
        }
    });

    Ok(json!({
        "session_id": session_id,
        "info_hash": info_hash,
        "name": metadata.name,
        "total_size": metadata.total_length,
    })
    .to_string())
}

/// Command to get download progress
#[tauri::command]
pub fn get_download_progress(session_id: String) -> Result<String, String> {
    let engines = DOWNLOAD_ENGINES.lock().unwrap();
    let engine = engines
        .get(&session_id)
        .ok_or_else(|| "Session not found".to_string())?;

    let stats = engine.stats();

    Ok(json!({
        "pieces": stats.pieces,
        "peers_connected": stats.peers_connected,
        "total_peers": stats.total_peers,
        "download_speed": stats.download_speed,
        "upload_speed": stats.upload_speed,
        "active_blocks": stats.active_blocks,
        "is_complete": engine.is_complete(),
        "session_id": session_id,
    })
    .to_string())
}

/// Command to get typed download stats (for API consumption)
#[tauri::command]
pub fn get_download_stats(session_id: String) -> Result<DownloadEngineStats, String> {
    let engines = DOWNLOAD_ENGINES.lock().unwrap();
    let engine = engines
        .get(&session_id)
        .ok_or_else(|| "Session not found".to_string())?;

    Ok(engine.stats())
}

/// Command to pause a download
#[tauri::command]
pub fn pause_download(session_id: String) -> Result<(), String> {
    let _engines = DOWNLOAD_ENGINES.lock().unwrap();
    tracing::info!("Pausing download session {}", session_id);
    // Implementation would pause active downloads
    Ok(())
}

/// Command to resume a download
#[tauri::command]
pub fn resume_download(session_id: String) -> Result<(), String> {
    let _engines = DOWNLOAD_ENGINES.lock().unwrap();
    tracing::info!("Resuming download session {}", session_id);
    // Implementation would resume paused downloads
    Ok(())
}

/// Command to cancel a download
#[tauri::command]
pub fn cancel_download(session_id: String) -> Result<(), String> {
    DOWNLOAD_ENGINES
        .lock()
        .unwrap()
        .remove(&session_id)
        .ok_or_else(|| "Session not found".to_string())?;

    tracing::info!("Cancelled download session {}", session_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use crate::modules::torrent_parser::TorrentMetadata;
    use crate::modules::download::DownloadSession;
    use crate::modules::pieces::PieceInfo;

    fn dummy_session() -> DownloadSession {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        DownloadSession {
            id: "test-session".to_string(),
            metadata: TorrentMetadata {
                info_hash: vec![0u8; 20],
                info_hash_hex: "0000000000000000000000000000000000000000".to_string(),
                name: "test".to_string(),
                total_length: 65536,
                piece_length: 16384,
                pieces_count: 4,
                pieces: vec![vec![0u8; 20]; 4],
                files: vec![crate::modules::torrent_parser::FileInfo {
                    path: vec!["test.dat".to_string()],
                    length: 65536,
                }],
                announce: "http://tracker.example.com".to_string(),
                announce_list: vec![],
                creation_date: None,
                comment: None,
            },
            started_at: now,
            uploaded: 0,
        }
    }

    #[test]
    fn should_store_and_retrieve_download_engine_stats() {
        let session = dummy_session();

        let pieces = (0..4)
            .map(|i| PieceInfo::new(i, vec![0u8; 20], 16384))
            .collect::<Vec<_>>();

        let engine = DownloadEngine::new(session.clone(), PathBuf::from("./tmp"), pieces);
        let session_id = session.id.clone();

        DOWNLOAD_ENGINES.lock().unwrap().insert(session_id.clone(), engine);

        let stats = get_download_stats(session_id.clone());

        assert!(stats.is_ok());
        let stats = stats.unwrap();
        assert_eq!(stats.peers_connected, 0);

        let _ = cancel_download(session_id);
    }
}
