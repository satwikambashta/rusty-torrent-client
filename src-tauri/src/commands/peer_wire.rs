use std::collections::HashMap;
use std::sync::Arc;
use tauri::State;

use crate::modules::peer_wire::{PeerWireProtocol, PeerMessage, PeerConnectionStats};
use crate::modules::peer::Peer;

/// Global state for peer wire protocol
pub struct PeerWireState {
    pub protocol: Arc<PeerWireProtocol>,
}

impl PeerWireState {
    pub fn new() -> Self {
        Self {
            protocol: Arc::new(PeerWireProtocol::new(50)), // Max 50 connections
        }
    }
}

/// Connect to a peer and perform handshake
#[tauri::command]
pub async fn connect_to_peer(
    peer: Peer,
    info_hash: Vec<u8>,
    peer_id: Vec<u8>,
    state: State<'_, PeerWireState>,
) -> Result<String, String> {
    if info_hash.len() != 20 || peer_id.len() != 20 {
        return Err("Invalid info_hash or peer_id length".to_string());
    }

    let mut info_hash_arr = [0u8; 20];
    let mut peer_id_arr = [0u8; 20];
    info_hash_arr.copy_from_slice(&info_hash);
    peer_id_arr.copy_from_slice(&peer_id);

    match state.protocol.connect_peer(peer, &info_hash_arr, &peer_id_arr).await {
        Ok(peer_key) => Ok(peer_key),
        Err(e) => Err(format!("Failed to connect to peer: {}", e)),
    }
}

/// Disconnect from a peer
#[tauri::command]
pub async fn disconnect_from_peer(
    peer_key: String,
    state: State<'_, PeerWireState>,
) -> Result<(), String> {
    match state.protocol.disconnect_peer(&peer_key).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to disconnect from peer: {}", e)),
    }
}

/// Send a choke message to a peer
#[tauri::command]
pub async fn choke_peer(
    peer_key: String,
    state: State<'_, PeerWireState>,
) -> Result<(), String> {
    match state.protocol.send_to_peer(&peer_key, PeerMessage::Choke).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to choke peer: {}", e)),
    }
}

/// Send an unchoke message to a peer
#[tauri::command]
pub async fn unchoke_peer(
    peer_key: String,
    state: State<'_, PeerWireState>,
) -> Result<(), String> {
    match state.protocol.send_to_peer(&peer_key, PeerMessage::Unchoke).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to unchoke peer: {}", e)),
    }
}

/// Send an interested message to a peer
#[tauri::command]
pub async fn express_interest(
    peer_key: String,
    state: State<'_, PeerWireState>,
) -> Result<(), String> {
    match state.protocol.send_to_peer(&peer_key, PeerMessage::Interested).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to express interest: {}", e)),
    }
}

/// Send a not interested message to a peer
#[tauri::command]
pub async fn express_not_interested(
    peer_key: String,
    state: State<'_, PeerWireState>,
) -> Result<(), String> {
    match state.protocol.send_to_peer(&peer_key, PeerMessage::NotInterested).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to express not interested: {}", e)),
    }
}

/// Request a block from a peer
#[tauri::command]
pub async fn request_piece_block(
    peer_key: String,
    piece_index: u32,
    begin: u32,
    length: u32,
    state: State<'_, PeerWireState>,
) -> Result<(), String> {
    match state.protocol.request_block(&peer_key, piece_index, begin, length).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to request block: {}", e)),
    }
}

/// Send a piece block to a peer
#[tauri::command]
pub async fn send_piece_block(
    peer_key: String,
    piece_index: u32,
    begin: u32,
    block: Vec<u8>,
    state: State<'_, PeerWireState>,
) -> Result<(), String> {
    match state.protocol.send_block(&peer_key, piece_index, begin, block).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to send block: {}", e)),
    }
}

/// Broadcast that we have a new piece to all peers
#[tauri::command]
pub async fn broadcast_have_piece(
    piece_index: u32,
    state: State<'_, PeerWireState>,
) -> Result<(), String> {
    match state.protocol.broadcast_have(piece_index).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to broadcast have: {}", e)),
    }
}

/// Receive pending messages from all connected peers
#[tauri::command]
pub async fn receive_peer_messages(
    state: State<'_, PeerWireState>,
) -> Result<Vec<(String, String)>, String> {
    match state.protocol.receive_messages().await {
        Ok(messages) => {
            let serialized_messages: Vec<(String, String)> = messages
                .into_iter()
                .map(|(peer_key, message)| {
                    let message_type = match message {
                        PeerMessage::KeepAlive => "keep_alive",
                        PeerMessage::Choke => "choke",
                        PeerMessage::Unchoke => "unchoke",
                        PeerMessage::Interested => "interested",
                        PeerMessage::NotInterested => "not_interested",
                        PeerMessage::Have { .. } => "have",
                        PeerMessage::Bitfield { .. } => "bitfield",
                        PeerMessage::Request { .. } => "request",
                        PeerMessage::Piece { .. } => "piece",
                        PeerMessage::Cancel { .. } => "cancel",
                        PeerMessage::Port { .. } => "port",
                    };
                    (peer_key, message_type.to_string())
                })
                .collect();
            Ok(serialized_messages)
        },
        Err(e) => Err(format!("Failed to receive messages: {}", e)),
    }
}

/// Get connection statistics for all peers
#[tauri::command]
pub async fn get_peer_connection_stats(
    state: State<'_, PeerWireState>,
) -> Result<HashMap<String, PeerConnectionStats>, String> {
    Ok(state.protocol.get_stats().await)
}

/// Check if a peer has a specific piece
#[tauri::command]
pub async fn peer_has_piece(
    peer_key: String,
    piece_index: u32,
    state: State<'_, PeerWireState>,
) -> Result<bool, String> {
    Ok(state.protocol.peer_has_piece(&peer_key, piece_index).await)
}

/// Get list of peers that have a specific piece
#[tauri::command]
pub async fn get_peers_with_piece(
    piece_index: u32,
    state: State<'_, PeerWireState>,
) -> Result<Vec<String>, String> {
    Ok(state.protocol.get_peers_with_piece(piece_index).await)
}