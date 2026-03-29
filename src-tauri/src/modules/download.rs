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

    /// Request blocks from peers
    pub fn request_blocks(
        &mut self,
        piece_index: u32,
    ) -> Result<Vec<(u32, u32, String)>, String> {
        // Get available peers for this piece
        let pool = self.peer_pool.lock().unwrap();
        let candidates: Vec<_> = pool
            .peers_with_piece(piece_index as usize)
            .iter()
            .filter(|p| p.state == crate::modules::peer::PeerState::Connected)
            .map(|p| p.addr.clone())
            .collect();

        drop(pool);

        if candidates.is_empty() {
            return Err("No peers have this piece".to_string());
        }

        // Create blocks and assign to peers
        let blocks = self.create_blocks(piece_index);
        let mut requests = Vec::new();

        for (i, block) in blocks.iter().enumerate() {
            let peer_idx = i % candidates.len();
            let peer_addr = candidates[peer_idx].clone();

            self.active_blocks
                .insert((piece_index, block.offset), peer_addr.clone());
            requests.push((piece_index, block.offset, peer_addr));
        }

        Ok(requests)
    }

    /// Save piece to disk
    pub fn save_piece(&mut self, piece_index: u32) -> Result<(), String> {
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
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create directory: {}", e))?;
            }

            // Write piece to file at correct offset
            use std::io::Seek;
            use std::io::Write;

            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&file_path)
                .map_err(|e| format!("Failed to open file: {}", e))?;

            file.seek(std::io::SeekFrom::Start(offset))
                .map_err(|e| format!("Failed to seek in file: {}", e))?;

            file.write_all(&data)
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
