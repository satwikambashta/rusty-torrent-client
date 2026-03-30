/// Download Engine Module
///
/// Core download orchestration
/// Manages:
/// - Connection to multiple peers
/// - Simultaneous piece downloads
/// - Block/chunk management
/// - File I/O and storage
/// - Progress tracking

use crate::modules::peer::{Peer, PeerPool};
use crate::modules::pieces::{DownloadProgress, PieceInfo, PieceState};
use crate::modules::torrent_parser::TorrentMetadata;
use crate::modules::peer_wire::{PeerWireProtocol, PeerMessage};
use serde::{Deserialize, Serialize};
use sha1::Digest;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Block (sub-piece) information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Piece index
    pub piece_index: u32,
    /// Offset within piece (bytes)
    pub offset: u32,
    /// Block size (bytes)
    pub size: u32,
    /// Is downloaded
    pub downloaded: bool,
}

impl Block {
    /// Create a new block
    pub fn new(piece_index: u32, offset: u32, size: u32) -> Self {
        Self {
            piece_index,
            offset,
            size,
            downloaded: false,
        }
    }
}

/// Download session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSession {
    /// Session ID
    pub id: String,
    /// Torrent metadata
    pub metadata: TorrentMetadata,
    /// Download start time
    pub started_at: i64,
    /// Total bytes uploaded (for seeding)
    pub uploaded: u64,
}

/// File piece mapping for multi-file torrents
/// Maps piece index to file and offset
#[derive(Debug, Clone)]
pub struct FilePieceMapping {
    /// Piece to (file_index, offset_in_file) mapping
    mappings: Vec<(usize, u64)>,
}

impl FilePieceMapping {
    /// Create mapping from torrent metadata
    pub fn from_metadata(metadata: &TorrentMetadata) -> Self {
        let mut mappings = Vec::new();
        let piece_length = metadata.piece_length;

        let mut current_offset = 0u64;
        for _ in 0..metadata.pieces_count {
            // Find which file this piece/offset belongs to
            let mut file_offset = current_offset;
            for (file_idx, file) in metadata.files.iter().enumerate() {
                if file_offset < file.length {
                    mappings.push((file_idx, file_offset));
                    break;
                }
                file_offset -= file.length;
            }
            current_offset += piece_length;
        }

        Self { mappings }
    }

    /// Get file and offset for a piece
    pub fn get_file_offset(&self, piece_index: usize) -> Option<(usize, u64)> {
        self.mappings.get(piece_index).copied()
    }
}

/// Download engine - orchestrates the download process
pub struct DownloadEngine {
    /// Current download session
    pub session: DownloadSession,
    /// Download directory
    pub download_dir: PathBuf,
    /// Peer pool
    pub peer_pool: Arc<Mutex<PeerPool>>,
    /// Peer wire protocol for communication
    pub peer_wire: Arc<PeerWireProtocol>,
    /// Download progress tracker
    pub progress: Arc<Mutex<DownloadProgress>>,
    /// File-piece mapping
    pub file_mapping: FilePieceMapping,
    /// Blocks being downloaded
    pub active_blocks: HashMap<(u32, u32), String>, // (piece_idx, offset) -> peer_addr
    /// Downloaded data buffer (piece_index -> data)
    pub piece_buffer: HashMap<u32, Vec<u8>>,
}

impl DownloadEngine {
    /// Create new download engine
    pub fn new(
        session: DownloadSession,
        download_dir: PathBuf,
        pieces: Vec<PieceInfo>,
    ) -> Self {
        let file_mapping = FilePieceMapping::from_metadata(&session.metadata);
        let progress = DownloadProgress::new(pieces);

        Self {
            session,
            download_dir,
            peer_pool: Arc::new(Mutex::new(PeerPool::new(200))),
            peer_wire: Arc::new(PeerWireProtocol::new(50)), // Max 50 peer connections
            progress: Arc::new(Mutex::new(progress)),
            file_mapping,
            active_blocks: HashMap::new(),
            piece_buffer: HashMap::new(),
        }
    }

    /// Add peer to pool
    pub fn add_peer(&self, peer: Peer) -> bool {
        let mut pool = self.peer_pool.lock().unwrap();
        pool.add_peer(peer)
    }

    /// Get next piece to download
    pub fn select_next_piece(&self) -> Option<u32> {
        let progress = self.progress.lock().unwrap();
        progress.select_next_piece()
    }

    /// Create blocks for a piece
    pub fn create_blocks(&self, piece_index: u32) -> Vec<Block> {
        let block_size = 16384u32; // Standard 16 KB blocks
        let piece_size = self.session.metadata.piece_length as u32;

        let mut blocks = Vec::new();
        let mut offset = 0u32;

        while offset < piece_size {
            let size = std::cmp::min(block_size, piece_size - offset);
            blocks.push(Block::new(piece_index, offset, size));
            offset += size;
        }

        blocks
    }

    /// Mark block as downloaded
    pub fn mark_block_downloaded(&mut self, piece_index: u32, offset: u32, data: Vec<u8>) {
        self.piece_buffer
            .entry(piece_index)
            .or_insert_with(Vec::new)
            .extend_from_slice(&data);

        self.active_blocks.remove(&(piece_index, offset));

        // Check if piece is complete
        let mut progress = self.progress.lock().unwrap();
        if let Some(piece) = progress.get_piece_mut(piece_index) {
            piece.downloaded += data.len() as u64;
            if piece.downloaded >= piece.size {
                piece.state = PieceState::Complete;
            }
        }
    }

    /// Connect to a peer using peer wire protocol
    pub async fn connect_to_peer(&self, peer: &Peer, peer_id: &[u8; 20]) -> Result<String, String> {
        let info_hash = self.session.metadata.info_hash.as_slice().try_into()
            .map_err(|_| "Invalid info hash length")?;

        self.peer_wire.connect_peer(peer.clone(), info_hash, peer_id).await
            .map_err(|e| format!("Failed to connect to peer: {}", e))
    }

    /// Disconnect from a peer
    pub async fn disconnect_from_peer(&self, peer_key: &str) -> Result<(), String> {
        self.peer_wire.disconnect_peer(peer_key).await
            .map_err(|e| format!("Failed to disconnect from peer: {}", e))
    }

    /// Request blocks from peers using peer wire protocol
    pub async fn request_blocks_from_peers(
        &mut self,
        piece_index: u32,
    ) -> Result<Vec<(u32, u32, String)>, String> {
        // Get peers that have this piece
        let peer_keys = self.peer_wire.get_peers_with_piece(piece_index).await;

        if peer_keys.is_empty() {
            return Err("No peers have this piece".to_string());
        }

        // Create blocks and assign to peers
        let blocks = self.create_blocks(piece_index);
        let mut requests = Vec::new();
        let _block_size = 16384u32; // Standard 16 KB blocks

        for (i, block) in blocks.iter().enumerate() {
            let peer_idx = i % peer_keys.len();
            let peer_key = &peer_keys[peer_idx];

            // Send request message
            self.peer_wire.request_block(peer_key.as_str(), piece_index, block.offset, block.size).await
                .map_err(|e| format!("Failed to request block: {}", e))?;

            self.active_blocks
                .insert((piece_index, block.offset), peer_key.clone());
            requests.push((piece_index, block.offset, peer_key.clone()));
        }

        Ok(requests)
    }

    /// Process incoming peer messages
    pub async fn process_peer_messages(&mut self) -> Result<Vec<String>, String> {
        let messages = self.peer_wire.receive_messages().await
            .map_err(|e| format!("Failed to receive messages: {}", e))?;

        let mut processed_events = Vec::new();

        for (peer_key, message) in messages {
            match message {
                PeerMessage::Piece { index, begin, block } => {
                    // Received a piece block
                    self.mark_block_downloaded(index, begin, block);
                    processed_events.push(format!("Received block {}:{} from {}", index, begin, peer_key));

                    // Check if piece is complete
                    if self.is_piece_complete(index) {
                        if let Err(e) = self.save_piece(index).await {
                            tracing::error!("Failed to save piece {}: {}", index, e);
                        } else {
                            // Broadcast that we have this piece
                            if let Err(e) = self.peer_wire.broadcast_have(index).await {
                                tracing::error!("Failed to broadcast have for piece {}: {}", index, e);
                            }
                            processed_events.push(format!("Completed and saved piece {}", index));
                        }
                    }
                },
                PeerMessage::Have { piece_index } => {
                    processed_events.push(format!("Peer {} has piece {}", peer_key, piece_index));
                },
                PeerMessage::Bitfield { bitfield: _ } => {
                    processed_events.push(format!("Received bitfield from peer {}", peer_key));
                },
                PeerMessage::Choke => {
                    processed_events.push(format!("Peer {} choked us", peer_key));
                },
                PeerMessage::Unchoke => {
                    processed_events.push(format!("Peer {} unchoked us", peer_key));
                },
                _ => {
                    // Other messages handled automatically by the protocol
                }
            }
        }

        Ok(processed_events)
    }

    /// Check if a piece is complete in the buffer
    pub fn is_piece_complete(&self, piece_index: u32) -> bool {
        if let Some(data) = self.piece_buffer.get(&piece_index) {
            let expected_size = self.session.metadata.piece_length as usize;
            // For the last piece, it might be smaller
            let actual_size = if piece_index as usize == self.session.metadata.pieces_count as usize - 1 {
                let total_size = self.session.metadata.total_length as usize;
                let piece_size = self.session.metadata.piece_length as usize;
                total_size % piece_size
            } else {
                expected_size
            };
            data.len() >= actual_size
        } else {
            false
        }
    }

    /// Save piece to disk (async version)
    pub async fn save_piece(&mut self, piece_index: u32) -> Result<(), String> {
        let data = self
            .piece_buffer
            .remove(&piece_index)
            .ok_or_else(|| "Piece data not in buffer".to_string())?;

        // Verify piece hash
        let mut hasher = sha1::Sha1::new();
        hasher.update(&data);
        let hash = hasher.finalize().to_vec();

        let progress = self.progress.lock().unwrap();
        if let Some(piece) = progress.get_piece(piece_index) {
            if piece.hash != hash.as_slice() {
                return Err(format!(
                    "Piece {} hash mismatch",
                    piece_index
                ));
            }
        }
        drop(progress);

        // Get file location for this piece
        if let Some((file_idx, offset)) = self.file_mapping.get_file_offset(piece_index as usize) {
            let file_path = self
                .download_dir
                .join(&self.session.metadata.files[file_idx].display_path());

            // Ensure parent directories exist
            if let Some(parent) = file_path.parent() {
                tokio::fs::create_dir_all(parent).await
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            // Write piece to file at correct offset
            use tokio::io::{AsyncSeekExt, AsyncWriteExt};

            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&file_path).await
                .map_err(|e| format!("Failed to open file: {}", e))?;

            file.seek(std::io::SeekFrom::Start(offset)).await
                .map_err(|e| format!("Failed to seek in file: {}", e))?;

            file.write_all(&data).await
                .map_err(|e| format!("Failed to write piece: {}", e))?;

            tracing::info!(
                "Saved piece {} to {:?} at offset {}",
                piece_index,
                file_path,
                offset
            );
            Ok(())
        } else {
            Err("File mapping not found for piece".to_string())
        }
    }

    /// Get download statistics
    pub fn stats(&self) -> DownloadEngineStats {
        let progress = self.progress.lock().unwrap();
        let pool = self.peer_pool.lock().unwrap();
        let peer_stats = pool.pool_stats();

        let progress_stats = progress.stats();

        DownloadEngineStats {
            pieces: progress_stats,
            peers_connected: peer_stats.connected_peers,
            total_peers: peer_stats.total_peers,
            download_speed: peer_stats.total_download_speed,
            upload_speed: peer_stats.total_upload_speed,
            active_blocks: self.active_blocks.len() as u32,
        }
    }

    /// Check if download is complete
    pub fn is_complete(&self) -> bool {
        let progress = self.progress.lock().unwrap();
        let stats = progress.stats();
        stats.completed_pieces >= stats.total_pieces
    }
}

/// Download engine statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadEngineStats {
    /// Pieces download statistics
    pub pieces: crate::modules::pieces::DownloadStats,
    /// Connected peers
    pub peers_connected: usize,
    /// Total peers in pool
    pub total_peers: usize,
    /// Download speed (bytes/sec)
    pub download_speed: u32,
    /// Upload speed (bytes/sec)
    pub upload_speed: u32,
    /// Currently active block requests
    pub active_blocks: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_session(total_size: u64) -> DownloadSession {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let pieces_count = ((total_size + 16383) / 16384) as u32;

        DownloadSession {
            id: uuid::Uuid::new_v4().to_string(),
            metadata: TorrentMetadata {
                info_hash: vec![1u8; 20],
                info_hash_hex: "0102030405060708090a0b0c0d0e0f1011121314".to_string(),
                name: "test".to_string(),
                total_length: total_size,
                piece_length: 16384,
                pieces_count,
                pieces: vec![vec![2u8; 20]; pieces_count as usize],
                files: vec![crate::modules::torrent_parser::FileInfo {
                    path: vec!["test.txt".to_string()],
                    length: total_size,
                }],
                announce: "http://tracker.example.com".to_string(),
                announce_list: vec![],
                creation_date: Some(now),
                comment: None,
            },
            started_at: now,
            uploaded: 0,
        }
    }

    #[test]
    fn test_create_engine() {
        let session = create_test_session(1024 * 1024);
        let pieces = vec![PieceInfo::new(0, vec![2u8; 20], 16384)];
        let engine = DownloadEngine::new(
            session,
            PathBuf::from("./downloads"),
            pieces,
        );
        assert!(!engine.session.id.is_empty());
    }

    #[test]
    fn test_create_blocks() {
        let session = create_test_session(32768);
        // create_blocks uses session.metadata.piece_length (16384)
        let pieces = vec![PieceInfo::new(0, vec![2u8; 20], 16384)];
        let engine = DownloadEngine::new(
            session,
            PathBuf::from("./downloads"),
            pieces,
        );

        let blocks = engine.create_blocks(0);
        assert_eq!(blocks.len(), 1); // 16KB / 16KB = 1 block
    }

    #[test]
    fn test_file_piece_mapping() {
        let session = create_test_session(100000);
        let mapping = FilePieceMapping::from_metadata(&session.metadata);
        let result = mapping.get_file_offset(0);
        assert!(result.is_some());
    }
}
