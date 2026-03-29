use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::modules::seeder::{SeederManager, SeedingConfig, SeedingStats, ChokingState};

/// Global seeder state
pub struct SeederState {
    pub manager: Arc<SeederManager>,
}

impl SeederState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(SeederManager::new(SeedingConfig::default())),
        }
    }
}

/// Register a peer for seeding
#[tauri::command]
pub async fn register_seeding_peer(
    peer_addr: String,
    state: State<'_, SeederState>,
) -> Result<(), String> {
    state.manager.register_peer(peer_addr).await
}

/// Unregister a seeding peer
#[tauri::command]
pub async fn unregister_seeding_peer(
    peer_addr: String,
    state: State<'_, SeederState>,
) -> Result<(), String> {
    state.manager.unregister_peer(&peer_addr).await
}

/// Mark peer as interested in our upload
#[tauri::command]
pub async fn seeding_peer_interested(
    peer_addr: String,
    state: State<'_, SeederState>,
) -> Result<(), String> {
    state.manager.peer_interested(&peer_addr).await
}

/// Request to upload a block
#[tauri::command]
pub async fn request_block_upload(
    peer_addr: String,
    block_size: u32,
    state: State<'_, SeederState>,
) -> Result<u32, String> {
    state.manager.request_block_upload(&peer_addr, block_size).await
}

/// Run choking algorithm manually
#[tauri::command]
pub async fn run_choking_algorithm(
    state: State<'_, SeederState>,
) -> Result<(), String> {
    state.manager.run_choking_algorithm().await
}

/// Update seeding configuration
#[tauri::command]
pub async fn update_seeding_config(
    max_upload_rate: u32,
    per_peer_limit: u32,
    max_uploading_peers: usize,
    state: State<'_, SeederState>,
) -> Result<(), String> {
    let mut config = SeedingConfig::default();
    config.max_upload_rate = max_upload_rate;
    config.per_peer_limit = per_peer_limit;
    config.max_uploading_peers = max_uploading_peers;
    state.manager.update_config(config).await
}

/// Get seeding statistics
#[tauri::command]
pub async fn get_seeding_stats(
    state: State<'_, SeederState>,
) -> Result<SeedingStats, String> {
    Ok(state.manager.get_stats().await)
}

/// Get list of seeding peers
#[tauri::command]
pub async fn get_seeding_peers(
    state: State<'_, SeederState>,
) -> Result<Vec<serde_json::Value>, String> {
    let peers = state.manager.get_peers().await;
    let mut result = Vec::new();

    for (addr, peer) in peers {
        let peer_json = serde_json::json!({
            "addr": addr,
            "choking": peer.choking_state == ChokingState::Choking,
            "uploaded": peer.uploaded,
            "downloaded": peer.downloaded,
            "peer_interested": peer.peer_interested,
            "upload_rate": peer.upload_rate,
        });
        result.push(peer_json);
    }

    Ok(result)
}

/// Clean up idle peers
#[tauri::command]
pub async fn cleanup_idle_seeding_peers(
    state: State<'_, SeederState>,
) -> Result<usize, String> {
    state.manager.cleanup_idle_peers().await
}