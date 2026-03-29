/// Peer Pool Management Module
///
/// Manages connections to peer nodes
/// Maintains peer metadata, status, and statistics
/// Implements peer selection and connection pooling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Peer state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeerState {
    /// Attempting to connect
    Connecting,
    /// Connected and handshake done
    Connected,
    /// Peer choked us
    Choked,
    /// We choked the peer
    Choking,
    /// Connection failed
    Failed,
    /// Peer disconnected
    Disconnected,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    /// Peer address (IP:port)
    pub addr: String,
    /// Peer ID (20 bytes, if available)
    pub peer_id: Option<Vec<u8>>,
    /// Current state
    pub state: PeerState,
    /// Upload speed (bytes/sec)
    pub upload_speed: u32,
    /// Download speed (bytes/sec)
    pub download_speed: u32,
    /// Pieces the peer has (bitfield)
    pub have_pieces: Vec<bool>,
    /// Is peer interested in us
    pub interested: bool,
    /// Connection time (Unix timestamp)
    pub connected_at: Option<i64>,
    /// Statistics
    pub stats: PeerStats,
}

/// Peer statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeerStats {
    /// Bytes uploaded to this peer
    pub uploaded: u64,
    /// Bytes downloaded from this peer
    pub downloaded: u64,
    /// Number of pieces received
    pub blocks_received: u64,
    /// Number of blocks sent
    pub blocks_sent: u64,
}

impl Peer {
    /// Create a new peer
    pub fn new(addr: String) -> Self {
        Self {
            addr,
            peer_id: None,
            state: PeerState::Connecting,
            upload_speed: 0,
            download_speed: 0,
            have_pieces: Vec::new(),
            interested: false,
            connected_at: None,
            stats: PeerStats {
                uploaded: 0,
                downloaded: 0,
                blocks_received: 0,
                blocks_sent: 0,
            },
        }
    }

    /// Mark peer as connected
    pub fn mark_connected(&mut self) {
        self.state = PeerState::Connected;
        self.connected_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs() as i64);
    }

    /// Mark peer as disconnected
    pub fn mark_disconnected(&mut self) {
        self.state = PeerState::Disconnected;
    }

    /// Update piece availability
    pub fn update_have_pieces(&mut self, pieces: Vec<bool>) {
        self.have_pieces = pieces;
    }

    /// Check if peer has a specific piece
    pub fn has_piece(&self, piece_index: usize) -> bool {
        self.have_pieces
            .get(piece_index)
            .copied()
            .unwrap_or(false)
    }

    /// Count pieces available at this peer
    pub fn piece_count(&self) -> u32 {
        self.have_pieces.iter().filter(|&&b| b).count() as u32
    }

    /// Update upload speed
    pub fn set_upload_speed(&mut self, speed: u32) {
        self.upload_speed = speed;
    }

    /// Update download speed
    pub fn set_download_speed(&mut self, speed: u32) {
        self.download_speed = speed;
    }

    /// Get connection duration in seconds
    pub fn connection_duration(&self) -> Option<i64> {
        self.connected_at.and_then(|connected| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as i64 - connected)
        })
    }
}

/// Peer pool for managing multiple peers
pub struct PeerPool {
    /// Peers by address
    peers: HashMap<String, Peer>,
    /// Maximum peers to keep
    max_peers: usize,
}

impl PeerPool {
    /// Create a new peer pool
    pub fn new(max_peers: usize) -> Self {
        Self {
            peers: HashMap::new(),
            max_peers,
        }
    }

    /// Add or update a peer
    pub fn add_peer(&mut self, peer: Peer) -> bool {
        if self.peers.len() >= self.max_peers && !self.peers.contains_key(&peer.addr) {
            // Pool is full, need to remove a peer first
            return false;
        }
        self.peers.insert(peer.addr.clone(), peer);
        true
    }

    /// Get a peer by address
    pub fn get_peer(&self, addr: &str) -> Option<&Peer> {
        self.peers.get(addr)
    }

    /// Get a mutable peer reference
    pub fn get_peer_mut(&mut self, addr: &str) -> Option<&mut Peer> {
        self.peers.get_mut(addr)
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, addr: &str) -> Option<Peer> {
        self.peers.remove(addr)
    }

    /// Get all peers
    pub fn all_peers(&self) -> Vec<&Peer> {
        self.peers.values().collect()
    }

    /// Get connected peers
    pub fn connected_peers(&self) -> Vec<&Peer> {
        self.peers
            .values()
            .filter(|p| p.state == PeerState::Connected)
            .collect()
    }

    /// Get interested in us peers (can upload to)
    pub fn interested_peers(&self) -> Vec<&Peer> {
        self.peers
            .values()
            .filter(|p| p.interested && p.state == PeerState::Connected)
            .collect()
    }

    /// Get peers that have a specific piece
    pub fn peers_with_piece(&self, piece_index: usize) -> Vec<&Peer> {
        self.peers
            .values()
            .filter(|p| p.has_piece(piece_index))
            .collect()
    }

    /// Get best upload peers (fastest)
    pub fn best_upload_peers(&self, count: usize) -> Vec<&Peer> {
        let mut peers = self.connected_peers();
        peers.sort_by_key(|p| std::cmp::Reverse(p.upload_speed));
        peers.into_iter().take(count).collect()
    }

    /// Get best download peers (fastest)
    pub fn best_download_peers(&self, count: usize) -> Vec<&Peer> {
        let mut peers = self.connected_peers();
        peers.sort_by_key(|p| std::cmp::Reverse(p.download_speed));
        peers.into_iter().take(count).collect()
    }

    /// Count peers in each state
    pub fn state_counts(&self) -> HashMap<PeerState, usize> {
        let mut counts = HashMap::new();
        for peer in self.peers.values() {
            *counts.entry(peer.state).or_insert(0) += 1;
        }
        counts
    }

    /// Get pool statistics
    pub fn pool_stats(&self) -> PoolStats {
        let peers = self.peers.values().collect::<Vec<_>>();
        let total_download_speed: u32 = peers.iter().map(|p| p.download_speed).sum();
        let total_upload_speed: u32 = peers.iter().map(|p| p.upload_speed).sum();

        let total_downloaded: u64 = peers.iter().map(|p| p.stats.downloaded).sum();
        let total_uploaded: u64 = peers.iter().map(|p| p.stats.uploaded).sum();

        PoolStats {
            total_peers: self.peers.len(),
            connected_peers: self
                .peers
                .values()
                .filter(|p| p.state == PeerState::Connected)
                .count(),
            total_download_speed,
            total_upload_speed,
            total_downloaded,
            total_uploaded,
        }
    }

    /// Clear disconnected peers
    pub fn cleanup_disconnected(&mut self) {
        self.peers
            .retain(|_, p| p.state != PeerState::Disconnected && p.state != PeerState::Failed);
    }

    /// Get peer count
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// Check if pool is empty
    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    /// Clear all peers
    pub fn clear(&mut self) {
        self.peers.clear();
    }
}

impl Default for PeerPool {
    fn default() -> Self {
        Self::new(200)
    }
}

/// Peer pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Total peers in pool
    pub total_peers: usize,
    /// Connected peers
    pub connected_peers: usize,
    /// Combined download speed
    pub total_download_speed: u32,
    /// Combined upload speed
    pub total_upload_speed: u32,
    /// Total bytes downloaded
    pub total_downloaded: u64,
    /// Total bytes uploaded
    pub total_uploaded: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_peer() {
        let peer = Peer::new("127.0.0.1:6881".to_string());
        assert_eq!(peer.addr, "127.0.0.1:6881");
        assert_eq!(peer.state, PeerState::Connecting);
        assert_eq!(peer.upload_speed, 0);
    }

    #[test]
    fn test_peer_connected() {
        let mut peer = Peer::new("127.0.0.1:6881".to_string());
        peer.mark_connected();
        assert_eq!(peer.state, PeerState::Connected);
        assert!(peer.connected_at.is_some());
    }

    #[test]
    fn test_peer_has_piece() {
        let mut peer = Peer::new("127.0.0.1:6881".to_string());
        peer.update_have_pieces(vec![true, false, true]);
        assert!(peer.has_piece(0));
        assert!(!peer.has_piece(1));
        assert!(peer.has_piece(2));
    }

    #[test]
    fn test_peer_piece_count() {
        let mut peer = Peer::new("127.0.0.1:6881".to_string());
        peer.update_have_pieces(vec![true, false, true, true, false]);
        assert_eq!(peer.piece_count(), 3);
    }

    #[test]
    fn test_peer_pool_add() {
        let mut pool = PeerPool::new(10);
        let peer = Peer::new("127.0.0.1:6881".to_string());
        assert!(pool.add_peer(peer));
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_peer_pool_get() {
        let mut pool = PeerPool::new(10);
        let peer = Peer::new("127.0.0.1:6881".to_string());
        pool.add_peer(peer);
        assert!(pool.get_peer("127.0.0.1:6881").is_some());
        assert!(pool.get_peer("127.0.0.2:6881").is_none());
    }

    #[test]
    fn test_peer_pool_connected() {
        let mut pool = PeerPool::new(10);
        let mut peer1 = Peer::new("127.0.0.1:6881".to_string());
        peer1.mark_connected();
        let peer2 = Peer::new("127.0.0.2:6881".to_string());
        pool.add_peer(peer1);
        pool.add_peer(peer2);
        assert_eq!(pool.connected_peers().len(), 1);
    }

    #[test]
    fn test_peer_pool_full() {
        let mut pool = PeerPool::new(2);
        pool.add_peer(Peer::new("127.0.0.1:6881".to_string()));
        pool.add_peer(Peer::new("127.0.0.2:6881".to_string()));
        let result = pool.add_peer(Peer::new("127.0.0.3:6881".to_string()));
        assert!(!result); // Pool is full
    }

    #[test]
    fn test_peer_pool_stats() {
        let mut pool = PeerPool::new(10);
        let mut peer = Peer::new("127.0.0.1:6881".to_string());
        peer.set_download_speed(100);
        peer.set_upload_speed(50);
        peer.stats.downloaded = 1024;
        peer.stats.uploaded = 512;
        pool.add_peer(peer);

        let stats = pool.pool_stats();
        assert_eq!(stats.total_download_speed, 100);
        assert_eq!(stats.total_upload_speed, 50);
    }

    #[test]
    fn test_peer_pool_state_counts() {
        let mut pool = PeerPool::new(10);
        let mut peer1 = Peer::new("127.0.0.1:6881".to_string());
        peer1.mark_connected();
        pool.add_peer(peer1);
        pool.add_peer(Peer::new("127.0.0.2:6881".to_string()));

        let counts = pool.state_counts();
        assert_eq!(counts.get(&PeerState::Connected), Some(&1));
        assert_eq!(counts.get(&PeerState::Connecting), Some(&1));
    }
}
