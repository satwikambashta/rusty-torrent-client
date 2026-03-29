/// Peer Discovery Command Handlers for Tauri IPC
///
/// Provides commands for:
/// - Discovering peers via DHT
/// - Announcing to trackers
/// - Managing peer connections

use crate::modules::dht::{DhtClient, DhtPeer};
use crate::modules::tracker::{HttpTracker, AnnounceRequest, TrackerEvent};
use crate::modules::peer::{PeerPool, Peer};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref DHT_CLIENT: Mutex<DhtClient> = Mutex::new(DhtClient::new());
    static ref PEER_POOL: Mutex<PeerPool> = Mutex::new(PeerPool::new(200));
}

/// DTO for peer information returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub addr: String,
    pub state: String,
    pub download_speed: u32,
    pub upload_speed: u32,
    pub piece_count: u32,
}

/// Command to discover peers for a torrent via DHT
#[tauri::command]
pub async fn discover_peers_dht(info_hash: String) -> Result<Vec<String>, String> {
    // Convert hex string to bytes
    let info_hash_bytes = hex::decode(&info_hash)
        .map_err(|e| format!("Invalid info hash: {}", e))?;

    let mut client = DHT_CLIENT.lock().unwrap();

    // Create get_peers query
    let query = client.create_get_peers_query(info_hash_bytes);

    // For now, return mock peers since we don't have actual UDP communication
    // In production, this would send the query to DHT nodes and parse responses
    let mock_peers = vec![
        "192.168.1.100:6881".to_string(),
        "192.168.1.101:6882".to_string(),
        "10.0.0.50:6881".to_string(),
    ];

    Ok(mock_peers)
}

/// Command to announce to a tracker and get peers
#[tauri::command]
pub async fn announce_to_tracker(
    announce_url: String,
    info_hash: String,
    peer_id: String,
    port: u16,
    uploaded: u64,
    downloaded: u64,
    left: u64,
    event: Option<String>,
) -> Result<Vec<String>, String> {
    // Convert hex strings to bytes
    let info_hash_bytes = hex::decode(&info_hash)
        .map_err(|e| format!("Invalid info hash: {}", e))?;
    let peer_id_bytes = hex::decode(&peer_id)
        .map_err(|e| format!("Invalid peer ID: {}", e))?;

    // Parse event
    let tracker_event = match event.as_deref() {
        Some("started") => Some(TrackerEvent::Started),
        Some("completed") => Some(TrackerEvent::Completed),
        Some("stopped") => Some(TrackerEvent::Stopped),
        _ => None,
    };

    // Create tracker client
    let tracker = HttpTracker::new(announce_url);

    // Build announce request
    let request = AnnounceRequest {
        info_hash: info_hash_bytes,
        peer_id: peer_id_bytes,
        port,
        uploaded,
        downloaded,
        left,
        event: tracker_event,
        ip: None,
        numwant: Some(50),
        key: None,
        trackerid: None,
        compact: true,
    };

    // Build announce URL
    let announce_url = tracker.build_announce_url(&request)
        .map_err(|e| format!("Failed to build announce URL: {}", e))?;

    // For now, return mock peers since we don't have actual HTTP requests
    // In production, this would make HTTP request to tracker and parse response
    let mock_peers = vec![
        "tracker-peer-1.example.com:6881".to_string(),
        "tracker-peer-2.example.com:6882".to_string(),
        "tracker-peer-3.example.com:6881".to_string(),
    ];

    Ok(mock_peers)
}

/// Command to add discovered peers to the peer pool
#[tauri::command]
pub fn add_discovered_peers(peer_addrs: Vec<String>) -> Result<(), String> {
    let mut pool = PEER_POOL.lock().unwrap();

    for addr in peer_addrs {
        let peer = Peer::new(addr);
        pool.add_peer(peer);
    }

    Ok(())
}

/// Command to get current peer pool status
#[tauri::command]
pub fn get_peer_pool_status() -> Result<serde_json::Value, String> {
    let pool = PEER_POOL.lock().unwrap();
    let stats = pool.pool_stats();

    let peers: Vec<PeerInfo> = pool.all_peers().into_iter().map(|p| PeerInfo {
        addr: p.addr.clone(),
        state: format!("{:?}", p.state).to_lowercase(),
        download_speed: p.download_speed,
        upload_speed: p.upload_speed,
        piece_count: p.piece_count(),
    }).collect();

    let status = serde_json::json!({
        "stats": stats,
        "peers": peers
    });

    Ok(status)
}

/// Command to connect to peers (placeholder for future implementation)
#[tauri::command]
pub async fn connect_to_peers(peer_addrs: Vec<String>) -> Result<(), String> {
    // For now, just add peers to pool
    // In production, this would establish actual TCP connections
    // and perform BitTorrent handshake protocol
    add_discovered_peers(peer_addrs)?;
    Ok(())
}