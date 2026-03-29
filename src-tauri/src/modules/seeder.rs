/// Seeding & Upload Module
///
/// Manages:
/// - Serving pieces to requesting peers
/// - Upload rate limiting (per-peer and global)
/// - Choking algorithm (peer selection for upload)
/// - Seeding statistics tracking
/// - Bandwidth management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use tokio::sync::Mutex;

/// Upload rate limiter with token bucket algorithm
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Maximum bytes per second
    max_bytes_per_sec: u32,
    /// Available tokens (bytes)
    tokens: Arc<Mutex<f64>>,
    /// Time of last update
    last_update: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter with max bytes per second
    pub fn new(max_bytes_per_sec: u32) -> Self {
        Self {
            max_bytes_per_sec,
            tokens: Arc::new(Mutex::new(max_bytes_per_sec as f64)),
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Check if we can upload `bytes` bytes
    pub async fn can_upload(&self, bytes: u32) -> bool {
        let mut tokens = self.tokens.lock().await;
        let mut last_update = self.last_update.lock().await;

        // Add tokens based on time elapsed
        let elapsed = last_update.elapsed().as_secs_f64();
        let new_tokens = (self.max_bytes_per_sec as f64) * elapsed;
        *tokens = (*tokens + new_tokens).min(self.max_bytes_per_sec as f64);
        *last_update = Instant::now();

        if *tokens >= bytes as f64 {
            *tokens -= bytes as f64;
            true
        } else {
            false
        }
    }

    /// Try to consume bytes, returns how many bytes can be uploaded
    pub async fn request_upload(&self, requested: u32) -> u32 {
        let mut tokens = self.tokens.lock().await;
        let mut last_update = self.last_update.lock().await;

        // Add tokens based on time elapsed
        let elapsed = last_update.elapsed().as_secs_f64();
        let new_tokens = (self.max_bytes_per_sec as f64) * elapsed;
        *tokens = (*tokens + new_tokens).min(self.max_bytes_per_sec as f64);
        *last_update = Instant::now();

        let available = (*tokens as u32).min(requested);
        *tokens -= available as f64;
        available
    }

    /// Get current available bytes
    pub async fn available_bytes(&self) -> u32 {
        let tokens = self.tokens.lock().await;
        *tokens as u32
    }
}

/// Peer choking state for seeding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChokingState {
    /// We are choking this peer (not uploading)
    Choking,
    /// We are not choking this peer (uploading)
    Unchoking,
}

/// Seeding peer state
#[derive(Debug, Clone)]
pub struct SeedingPeer {
    /// Peer address (IP:port)
    pub peer_addr: String,
    /// Choking state
    pub choking_state: ChokingState,
    /// Bytes uploaded in current session
    pub uploaded: u64,
    /// Bytes downloaded (for reciprocity)
    pub downloaded: u64,
    /// Is peer interested in our upload
    pub peer_interested: bool,
    /// Upload rate (bytes/sec)
    pub upload_rate: f64,
    /// Last activity time
    pub last_activity: i64,
    /// Individual rate limiter for this peer
    pub rate_limiter: RateLimiter,
}

impl SeedingPeer {
    /// Create a new seeding peer
    pub fn new(peer_addr: String, rate_limit: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Self {
            peer_addr,
            choking_state: ChokingState::Choking,
            uploaded: 0,
            downloaded: 0,
            peer_interested: false,
            upload_rate: 0.0,
            last_activity: now,
            rate_limiter: RateLimiter::new(rate_limit),
        }
    }

    /// Record upload to this peer
    pub fn record_upload(&mut self, bytes: u64) {
        self.uploaded += bytes;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        self.last_activity = now;

        // Update upload rate (simple moving average)
        self.upload_rate = (self.upload_rate * 0.7) + (bytes as f64 * 0.3);
    }

    /// Check if peer is idle (no activity for timeout seconds)
    pub fn is_idle(&self, timeout_secs: i64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        now - self.last_activity > timeout_secs
    }
}

/// Global seeding statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedingStats {
    /// Total bytes uploaded across all peers
    pub total_uploaded: u64,
    /// Total bytes downloaded
    pub total_downloaded: u64,
    /// Number of active seeding peers
    pub active_peers: usize,
    /// Number of choked peers
    pub choked_peers: usize,
    /// Average upload rate (bytes/sec)
    pub avg_upload_rate: f64,
    /// Peak upload rate (bytes/sec)
    pub peak_upload_rate: f64,
    /// Number of block requests served
    pub blocks_served: u64,
    /// Session start time (unix timestamp)
    pub session_start: i64,
}

impl Default for SeedingStats {
    fn default() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        Self {
            total_uploaded: 0,
            total_downloaded: 0,
            active_peers: 0,
            choked_peers: 0,
            avg_upload_rate: 0.0,
            peak_upload_rate: 0.0,
            blocks_served: 0,
            session_start: now,
        }
    }
}

/// Seeding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedingConfig {
    /// Global upload rate limit (bytes/sec), 0 = unlimited
    pub max_upload_rate: u32,
    /// Per-peer upload rate limit (bytes/sec)
    pub per_peer_limit: u32,
    /// Maximum number of unchoking peers
    pub max_uploading_peers: usize,
    /// Rechoking interval (seconds)
    pub rechoking_interval: u64,
    /// Optimistic unchoke interval (seconds)
    pub optimistic_unchoke_interval: u64,
    /// Peer idle timeout (seconds)
    pub peer_idle_timeout: i64,
}

impl Default for SeedingConfig {
    fn default() -> Self {
        Self {
            max_upload_rate: 0, // Unlimited
            per_peer_limit: 1024 * 1024, // 1 MB/s per peer
            max_uploading_peers: 4,
            rechoking_interval: 10,
            optimistic_unchoke_interval: 30,
            peer_idle_timeout: 120,
        }
    }
}

/// Seeding manager
pub struct SeederManager {
    /// Configuration
    pub config: Arc<Mutex<SeedingConfig>>,
    /// Connected seeding peers
    peers: Arc<Mutex<HashMap<String, SeedingPeer>>>,
    /// Statistics
    stats: Arc<Mutex<SeedingStats>>,
    /// Global rate limiter
    global_limiter: RateLimiter,
    /// Last rechoking time
    last_rechoking: Arc<Mutex<Instant>>,
    /// Last optimistic unchoke time
    last_optimistic_unchoke: Arc<Mutex<Instant>>,
}

impl SeederManager {
    /// Create a new seeder manager
    pub fn new(config: SeedingConfig) -> Self {
        let limiter = if config.max_upload_rate > 0 {
            RateLimiter::new(config.max_upload_rate)
        } else {
            RateLimiter::new(u32::MAX)
        };

        Self {
            config: Arc::new(Mutex::new(config)),
            peers: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(SeedingStats::default())),
            global_limiter: limiter,
            last_rechoking: Arc::new(Mutex::new(Instant::now())),
            last_optimistic_unchoke: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Register a seeding peer
    pub async fn register_peer(&self, peer_addr: String) -> Result<(), String> {
        let config = self.config.lock().await;
        let per_peer_limit = config.per_peer_limit;
        drop(config);

        let peer = SeedingPeer::new(peer_addr.clone(), per_peer_limit);
        let mut peers = self.peers.lock().await;

        if peers.len() >= 100 {
            return Err("Too many peers registered".to_string());
        }

        peers.insert(peer_addr, peer);
        Ok(())
    }

    /// Unregister a seeding peer
    pub async fn unregister_peer(&self, peer_addr: &str) -> Result<(), String> {
        let mut peers = self.peers.lock().await;
        peers.remove(peer_addr);
        Ok(())
    }

    /// Mark peer as interested
    pub async fn peer_interested(&self, peer_addr: &str) -> Result<(), String> {
        let mut peers = self.peers.lock().await;
        if let Some(peer) = peers.get_mut(peer_addr) {
            peer.peer_interested = true;
            Ok(())
        } else {
            Err(format!("Peer {} not found", peer_addr))
        }
    }

    /// Request to upload block to peer
    pub async fn request_block_upload(
        &self,
        peer_addr: &str,
        block_size: u32,
    ) -> Result<u32, String> {
        // Check global rate limit
        let global_available = self.global_limiter.request_upload(block_size).await;

        if global_available == 0 {
            return Ok(0); // Rate limited globally
        }

        // Check peer rate limit
        let mut peers = self.peers.lock().await;
        if let Some(peer) = peers.get_mut(peer_addr) {
            if peer.choking_state == ChokingState::Choking {
                return Err(format!("Peer {} is choked", peer_addr));
            }

            let peer_available = peer.rate_limiter.request_upload(global_available).await;
            peer.record_upload(peer_available as u64);

            // Update statistics
            let mut stats = self.stats.lock().await;
            stats.total_uploaded += peer_available as u64;
            stats.blocks_served += 1;
            stats.avg_upload_rate = (stats.avg_upload_rate * 0.9) + (peer_available as f64 * 0.1);
            if stats.avg_upload_rate > stats.peak_upload_rate {
                stats.peak_upload_rate = stats.avg_upload_rate;
            }

            Ok(peer_available)
        } else {
            Err(format!("Peer {} not found", peer_addr))
        }
    }

    /// Run choking algorithm (should be called periodically)
    pub async fn run_choking_algorithm(&self) -> Result<(), String> {
        let config = self.config.lock().await;
        let max_uploading = config.max_uploading_peers;
        let rechoking_interval = config.rechoking_interval;
        let optimistic_interval = config.optimistic_unchoke_interval;
        drop(config);

        let now = Instant::now();
        let mut last_rechoking = self.last_rechoking.lock().await;
        let mut last_optimistic = self.last_optimistic_unchoke.lock().await;

        // Regular rechoking every N seconds
        if last_rechoking.elapsed() > Duration::from_secs(rechoking_interval) {
            self.rechoking_round(max_uploading).await?;
            *last_rechoking = now;
        }

        // Optimistic unchoke every N seconds
        if last_optimistic.elapsed() > Duration::from_secs(optimistic_interval) {
            self.optimistic_unchoke().await?;
            *last_optimistic = now;
        }

        Ok(())
    }

    /// Standard bitTorrent rechoking: unchoke fastest uploaders
    async fn rechoking_round(&self, max_uploading: usize) -> Result<(), String> {
        let mut peers = self.peers.lock().await;

        // Find interested peers sorted by reciprocity (download ratio)
        let mut interested_peers: Vec<_> = peers
            .iter_mut()
            .filter(|(_, p)| p.peer_interested)
            .collect();

        // Sort by download/upload ratio (favor those who uploaded to us)
        interested_peers.sort_by(|a, b| {
            let ratio_a = if a.1.uploaded > 0 {
                a.1.downloaded as f64 / a.1.uploaded as f64
            } else {
                0.0
            };
            let ratio_b = if b.1.uploaded > 0 {
                b.1.downloaded as f64 / b.1.uploaded as f64
            } else {
                0.0
            };
            ratio_b.partial_cmp(&ratio_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Unchoke top N, choke others
        for (idx, (_, peer)) in interested_peers.iter_mut().enumerate() {
            if idx < max_uploading {
                peer.choking_state = ChokingState::Unchoking;
            } else {
                peer.choking_state = ChokingState::Choking;
            }
        }

        Ok(())
    }

    /// Optimistic unchoke: give chance to one random peer
    async fn optimistic_unchoke(&self) -> Result<(), String> {
        let mut peers = self.peers.lock().await;

        // Find a choked peer that's interested
        for peer in peers.values_mut() {
            if peer.choking_state == ChokingState::Choking && peer.peer_interested {
                peer.choking_state = ChokingState::Unchoking;
                break;
            }
        }

        Ok(())
    }

    /// Update configuration
    pub async fn update_config(&self, config: SeedingConfig) -> Result<(), String> {
        let mut current_config = self.config.lock().await;
        *current_config = config;
        Ok(())
    }

    /// Get seeding statistics
    pub async fn get_stats(&self) -> SeedingStats {
        let mut stats = self.stats.lock().await;
        stats.active_peers = self.get_active_peers_count().await;
        stats.choked_peers = self.get_choked_peers_count().await;
        stats.clone()
    }

    /// Get list of all seeding peers
    pub async fn get_peers(&self) -> Vec<(String, SeedingPeer)> {
        let peers = self.peers.lock().await;
        peers.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// Clean up idle peers
    pub async fn cleanup_idle_peers(&self) -> Result<usize, String> {
        let config = self.config.lock().await;
        let timeout = config.peer_idle_timeout;
        drop(config);

        let mut peers = self.peers.lock().await;
        let initial_count = peers.len();

        peers.retain(|_, peer| !peer.is_idle(timeout));

        let removed = initial_count - peers.len();
        Ok(removed as usize)
    }

    /// Get number of active (unchoking) peers
    async fn get_active_peers_count(&self) -> usize {
        let peers = self.peers.lock().await;
        peers
            .values()
            .filter(|p| p.choking_state == ChokingState::Unchoking)
            .count()
    }

    /// Get number of choked peers
    async fn get_choked_peers_count(&self) -> usize {
        let peers = self.peers.lock().await;
        peers
            .values()
            .filter(|p| p.choking_state == ChokingState::Choking)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(1024);
        assert_eq!(limiter.max_bytes_per_sec, 1024);
    }

    #[tokio::test]
    async fn test_rate_limiter_upload() {
        let limiter = RateLimiter::new(1024);
        
        // First upload should succeed (within initial tokens)
        let available = limiter.request_upload(512).await;
        assert_eq!(available, 512);

        // Wait a bit for tokens to refill
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Second upload should work after some time
        let available2 = limiter.request_upload(512).await;
        assert!(available2 > 0);
    }

    #[test]
    fn test_seeding_peer_creation() {
        let peer = SeedingPeer::new("127.0.0.1:6881".to_string(), 1024);
        assert_eq!(peer.choking_state, ChokingState::Choking);
        assert_eq!(peer.uploaded, 0);
        assert!(!peer.peer_interested);
    }

    #[test]
    fn test_seeding_stats_default() {
        let stats = SeedingStats::default();
        assert_eq!(stats.total_uploaded, 0);
        assert_eq!(stats.active_peers, 0);
    }

    #[tokio::test]
    async fn test_seeder_manager_register_peer() {
        let config = SeedingConfig::default();
        let manager = SeederManager::new(config);

        assert!(manager.register_peer("127.0.0.1:6881".to_string()).await.is_ok());
    }

    #[tokio::test]
    async fn test_seeder_manager_peer_interested() {
        let config = SeedingConfig::default();
        let manager = SeederManager::new(config);

        manager.register_peer("127.0.0.1:6881".to_string()).await.unwrap();
        assert!(manager.peer_interested("127.0.0.1:6881").await.is_ok());

        let peers = manager.get_peers().await;
        assert_eq!(peers[0].1.peer_interested, true);
    }

    #[tokio::test]
    async fn test_seeder_manager_stats() {
        let config = SeedingConfig::default();
        let manager = SeederManager::new(config);

        manager.register_peer("127.0.0.1:6881".to_string()).await.unwrap();

        let stats = manager.get_stats().await;
        assert_eq!(stats.total_uploaded, 0);
    }
}