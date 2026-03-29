/// Download Command Handlers for Tauri IPC
///
/// Provides commands for:
/// - Loading torrent files
/// - Starting/pausing downloads
/// - Getting download progress
/// - Managing download sessions

use crate::modules::download::DownloadEngine;
use crate::modules::pieces::PieceInfo;
use crate::modules::torrent_parser::TorrentParser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref DOWNLOAD_ENGINES: Mutex<std::collections::HashMap<String, DownloadEngine>> = 
        Mutex::new(std::collections::HashMap::new());
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
    })
    .to_string())
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
