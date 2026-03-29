/// Piece Selection Module
///
/// Implements piece selection algorithms for optimal download strategy
/// Strategies: Rarest First, Sequential, Random, End-game

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Piece state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PieceState {
    /// Not yet downloaded
    Missing,
    /// Partially downloaded
    Partial,
    /// Fully downloaded
    Complete,
}

/// Piece information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieceInfo {
    /// Piece index
    pub index: u32,
    /// Hash of the piece (20 bytes)
    pub hash: Vec<u8>,
    /// Size in bytes
    pub size: u64,
    /// Download state
    pub state: PieceState,
    /// Bytes downloaded
    pub downloaded: u64,
    /// Availability (number of peers with this piece)
    pub availability: u32,
    /// Priority (higher = download first)
    pub priority: u8,
}

impl PieceInfo {
    /// Create a new piece
    pub fn new(index: u32, hash: Vec<u8>, size: u64) -> Self {
        Self {
            index,
            hash,
            size,
            state: PieceState::Missing,
            downloaded: 0,
            availability: 0,
            priority: 128, // Default priority (middle value)
        }
    }

    /// Get download progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.size == 0 {
            0.0
        } else {
            (self.downloaded as f32) / (self.size as f32)
        }
    }

    /// Mark as complete
    pub fn mark_complete(&mut self) {
        if self.downloaded >= self.size {
            self.state = PieceState::Complete;
        }
    }

    /// Set availability count
    pub fn set_availability(&mut self, count: u32) {
        self.availability = count;
    }

    /// Check if piece is fully downloaded
    pub fn is_complete(&self) -> bool {
        self.state == PieceState::Complete || self.downloaded >= self.size
    }
}

/// Piece selection strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SelectionStrategy {
    /// Download rarest pieces first (avoids getting stuck)
    RarestFirst,
    /// Download sequentially from beginning
    Sequential,
    /// Download in random order
    Random,
    /// Download any available piece (used in endgame)
    EndGame,
    /// High priority pieces first
    Priority,
}

/// Download progress tracker
pub struct DownloadProgress {
    /// All pieces
    pieces: Vec<PieceInfo>,
    /// Current selection strategy
    strategy: SelectionStrategy,
    /// Currently downloading piece indices
    downloading: HashSet<u32>,
}

impl DownloadProgress {
    /// Create new progress tracker
    pub fn new(pieces: Vec<PieceInfo>) -> Self {
        Self {
            pieces,
            strategy: SelectionStrategy::RarestFirst,
            downloading: HashSet::new(),
        }
    }

    /// Get piece by index
    pub fn get_piece(&self, index: u32) -> Option<&PieceInfo> {
        self.pieces.iter().find(|p| p.index == index)
    }

    /// Get mutable piece reference
    pub fn get_piece_mut(&mut self, index: u32) -> Option<&mut PieceInfo> {
        self.pieces.iter_mut().find(|p| p.index == index)
    }

    /// Get all pieces
    pub fn pieces(&self) -> &[PieceInfo] {
        &self.pieces
    }

    /// Set selection strategy
    pub fn set_strategy(&mut self, strategy: SelectionStrategy) {
        self.strategy = strategy;
    }

    /// Update piece availability from peer bitfield
    /// bitfield: vector of booleans indicating which pieces peer has
    pub fn update_availability(&mut self, bitfield: &[bool]) {
        for (piece, &has_piece) in self.pieces.iter_mut().zip(bitfield.iter()) {
            if has_piece {
                piece.availability += 1;
            }
        }
    }

    /// Select next piece to download
    pub fn select_next_piece(&self) -> Option<u32> {
        match self.strategy {
            SelectionStrategy::RarestFirst => self.select_rarest_first(),
            SelectionStrategy::Sequential => self.select_sequential(),
            SelectionStrategy::Random => self.select_random(),
            SelectionStrategy::EndGame => self.select_endgame(),
            SelectionStrategy::Priority => self.select_by_priority(),
        }
    }

    /// Rarest first strategy - minimizes risk of getting stuck
    fn select_rarest_first(&self) -> Option<u32> {
        self.pieces
            .iter()
            .filter(|p| {
                p.state != PieceState::Complete
                    && !self.downloading.contains(&p.index)
                    && p.availability > 0
            })
            .min_by_key(|p| p.availability)
            .map(|p| p.index)
    }

    /// Sequential strategy - download in order
    fn select_sequential(&self) -> Option<u32> {
        self.pieces
            .iter()
            .find(|p| {
                p.state != PieceState::Complete && !self.downloading.contains(&p.index)
            })
            .map(|p| p.index)
    }

    /// Random strategy - download in random order
    fn select_random(&self) -> Option<u32> {
        self.pieces
            .iter()
            .find(|p| {
                p.state != PieceState::Complete
                    && !self.downloading.contains(&p.index)
                    && p.availability > 0
            })
            .map(|p| p.index)
    }

    /// Endgame strategy - request from multiple peers simultaneously
    fn select_endgame(&self) -> Option<u32> {
        self.pieces
            .iter()
            .find(|p| p.state == PieceState::Partial)
            .map(|p| p.index)
    }

    /// Priority strategy - download high priority pieces first
    fn select_by_priority(&self) -> Option<u32> {
        self.pieces
            .iter()
            .filter(|p| {
                p.state != PieceState::Complete && !self.downloading.contains(&p.index)
            })
            .max_by_key(|p| p.priority)
            .map(|p| p.index)
    }

    /// Mark piece as downloading
    pub fn start_downloading(&mut self, piece_index: u32) -> bool {
        if let Some(piece) = self.get_piece_mut(piece_index) {
            if piece.state != PieceState::Complete && !self.downloading.contains(&piece_index) {
                self.downloading.insert(piece_index);
                return true;
            }
        }
        false
    }

    /// Mark piece as no longer downloading
    pub fn stop_downloading(&mut self, piece_index: u32) {
        self.downloading.remove(&piece_index);
    }

    /// Update downloaded bytes for a piece
    pub fn update_piece_progress(&mut self, piece_index: u32, bytes: u64) {
        if let Some(piece) = self.get_piece_mut(piece_index) {
            piece.downloaded += bytes;
            if piece.downloaded >= piece.size {
                piece.state = PieceState::Complete;
            } else if piece.downloaded > 0 {
                piece.state = PieceState::Partial;
            }
        }
    }

    /// Get download statistics
    pub fn stats(&self) -> DownloadStats {
        let total_pieces = self.pieces.len();
        let completed = self.pieces.iter().filter(|p| p.is_complete()).count();
        let total_size: u64 = self.pieces.iter().map(|p| p.size).sum();
        let downloaded: u64 = self.pieces.iter().map(|p| p.downloaded).sum();

        DownloadStats {
            total_pieces: total_pieces as u32,
            completed_pieces: completed as u32,
            total_size,
            downloaded_size: downloaded,
            progress: if total_size > 0 {
                (downloaded as f32 / total_size as f32) * 100.0
            } else {
                0.0
            },
            currently_downloading: self.downloading.len() as u32,
        }
    }

    /// Get pieces statistics (rarity, completion, etc.)
    pub fn pieces_stats(&self) -> PiecesStats {
        let mut by_state: HashMap<PieceState, u32> = HashMap::new();
        let mut total_availability = 0u32;
        let mut min_availability = u32::MAX;
        let mut max_availability = 0u32;

        for piece in &self.pieces {
            *by_state.entry(piece.state).or_insert(0) += 1;
            total_availability += piece.availability;
            min_availability = min_availability.min(piece.availability);
            max_availability = max_availability.max(piece.availability);
        }

        PiecesStats {
            by_state,
            avg_availability: if self.pieces.is_empty() {
                0.0
            } else {
                total_availability as f32 / self.pieces.len() as f32
            },
            min_availability,
            max_availability,
        }
    }
}

/// Download statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStats {
    /// Total pieces in torrent
    pub total_pieces: u32,
    /// Completed pieces
    pub completed_pieces: u32,
    /// Total size in bytes
    pub total_size: u64,
    /// Downloaded size in bytes
    pub downloaded_size: u64,
    /// Progress percentage (0-100)
    pub progress: f32,
    /// Pieces currently downloading
    pub currently_downloading: u32,
}

/// Pieces statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiecesStats {
    /// Count by state
    pub by_state: HashMap<PieceState, u32>,
    /// Average availability
    pub avg_availability: f32,
    /// Minimum availability
    pub min_availability: u32,
    /// Maximum availability
    pub max_availability: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pieces(count: u32) -> Vec<PieceInfo> {
        (0..count)
            .map(|i| PieceInfo::new(i, vec![i as u8; 20], 16384))
            .collect()
    }

    #[test]
    fn test_piece_creation() {
        let piece = PieceInfo::new(0, vec![0u8; 20], 1024);
        assert_eq!(piece.index, 0);
        assert_eq!(piece.size, 1024);
        assert_eq!(piece.state, PieceState::Missing);
    }

    #[test]
    fn test_piece_progress() {
        let mut piece = PieceInfo::new(0, vec![0u8; 20], 1000);
        piece.downloaded = 500;
        assert_eq!(piece.progress(), 0.5);
    }

    #[test]
    fn test_download_progress_creation() {
        let pieces = create_test_pieces(10);
        let progress = DownloadProgress::new(pieces);
        assert_eq!(progress.pieces().len(), 10);
    }

    #[test]
    fn test_select_rarest_first() {
        let mut pieces = create_test_pieces(3);
        pieces[0].availability = 10;
        pieces[1].availability = 1;
        pieces[2].availability = 5;

        let mut progress = DownloadProgress::new(pieces);
        progress.set_strategy(SelectionStrategy::RarestFirst);

        let next = progress.select_next_piece();
        assert_eq!(next, Some(1)); // Rarest: availability 1
    }

    #[test]
    fn test_select_sequential() {
        let pieces = create_test_pieces(3);
        let mut progress = DownloadProgress::new(pieces);
        progress.set_strategy(SelectionStrategy::Sequential);

        let next = progress.select_next_piece();
        assert_eq!(next, Some(0)); // First piece
    }

    #[test]
    fn test_start_downloading() {
        let mut pieces = create_test_pieces(3);
        // Set availability for rarest first strategy
        pieces[0].availability = 5;
        pieces[1].availability = 3;
        pieces[2].availability = 2;

        let mut progress = DownloadProgress::new(pieces);

        assert!(progress.start_downloading(0));
        assert!(!progress.start_downloading(0)); // Already downloading

        let next = progress.select_next_piece();
        assert_eq!(next, Some(2)); // Rarest (availability 2)
    }

    #[test]
    fn test_update_availability() {
        let pieces = create_test_pieces(3);
        let mut progress = DownloadProgress::new(pieces);

        let bitfield = vec![true, false, true];
        progress.update_availability(&bitfield);

        assert_eq!(progress.get_piece(0).unwrap().availability, 1);
        assert_eq!(progress.get_piece(1).unwrap().availability, 0);
        assert_eq!(progress.get_piece(2).unwrap().availability, 1);
    }

    #[test]
    fn test_download_stats() {
        let mut pieces = create_test_pieces(3);
        pieces[0].downloaded = 16384;
        pieces[0].mark_complete();
        pieces[1].downloaded = 8192;
        pieces[1].state = PieceState::Partial;

        let progress = DownloadProgress::new(pieces);
        let stats = progress.stats();

        assert_eq!(stats.total_pieces, 3);
        assert_eq!(stats.completed_pieces, 1);
        assert_eq!(stats.currently_downloading, 0);
    }

    #[test]
    fn test_priority_strategy() {
        let mut pieces = create_test_pieces(3);
        pieces[0].priority = 50;
        pieces[1].priority = 200;
        pieces[2].priority = 100;

        let mut progress = DownloadProgress::new(pieces);
        progress.set_strategy(SelectionStrategy::Priority);

        let next = progress.select_next_piece();
        assert_eq!(next, Some(1)); // Highest priority
    }

    #[test]
    fn test_endgame_strategy() {
        let mut pieces = create_test_pieces(3);
        pieces[1].state = PieceState::Partial;
        pieces[1].downloaded = 5000;

        let mut progress = DownloadProgress::new(pieces);
        progress.set_strategy(SelectionStrategy::EndGame);

        let next = progress.select_next_piece();
        assert_eq!(next, Some(1)); // Partial piece in endgame
    }
}
