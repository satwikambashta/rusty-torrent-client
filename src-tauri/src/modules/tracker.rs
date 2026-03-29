/// Tracker Communication Module
///
/// Handles communication with torrent trackers (HTTP/UDP)
/// Sends announce requests and processes peer lists
/// Implements BEP 3 - Tracker Protocol (HTTP)

use crate::modules::parser::BencodeParser;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Tracker announce event types
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrackerEvent {
    /// Started downloading
    Started,
    /// Completed torrent
    Completed,
    /// Stopped downloading
    Stopped,
}

impl fmt::Display for TrackerEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackerEvent::Started => write!(f, "started"),
            TrackerEvent::Completed => write!(f, "completed"),
            TrackerEvent::Stopped => write!(f, "stopped"),
        }
    }
}

/// Tracker announce request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceRequest {
    /// Info hash (20 bytes)
    pub info_hash: Vec<u8>,
    /// Peer ID (20 bytes)
    pub peer_id: Vec<u8>,
    /// Listen port
    pub port: u16,
    /// Uploaded bytes
    pub uploaded: u64,
    /// Downloaded bytes
    pub downloaded: u64,
    /// Remaining bytes to download
    pub left: u64,
    /// Event (optional)
    pub event: Option<TrackerEvent>,
    /// IP address to report (optional)
    pub ip: Option<String>,
    /// Number of peers to request
    pub numwant: Option<u32>,
    /// Tracker key (optional)
    pub key: Option<String>,
    /// Tracker ID (optional)
    pub trackerid: Option<String>,
    /// Compact peer list format
    pub compact: bool,
}

/// Tracker peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerPeer {
    /// Peer ID (optional)
    pub peer_id: Option<String>,
    /// Peer IP address
    pub ip: String,
    /// Peer port
    pub port: u16,
}

/// Tracker announce response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnounceResponse {
    /// Interval to wait before next announce
    pub interval: u32,
    /// Minimum interval between announces
    pub min_interval: Option<u32>,
    /// Tracker ID (returned by tracker)
    pub trackerid: Option<String>,
    /// Number of seeders
    pub complete: u32,
    /// Number of leechers
    pub incomplete: u32,
    /// List of peers
    pub peers: Vec<TrackerPeer>,
}

/// Tracker error
#[derive(Debug)]
pub enum TrackerError {
    /// HTTP error response
    HttpError(String),
    /// Invalid response format
    InvalidFormat(String),
    /// Network error
    NetworkError(String),
    /// Tracker error message
    TrackerError(String),
}

impl fmt::Display for TrackerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrackerError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            TrackerError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            TrackerError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            TrackerError::TrackerError(msg) => write!(f, "Tracker error: {}", msg),
        }
    }
}

impl std::error::Error for TrackerError {}

/// HTTP Tracker client
pub struct HttpTracker {
    announce_url: String,
}

impl HttpTracker {
    /// Create a new tracker client
    pub fn new(announce_url: String) -> Self {
        Self { announce_url }
    }

    /// Get announce URL
    pub fn announce_url(&self) -> &str {
        &self.announce_url
    }

    /// Build tracker announce URL with parameters
    pub fn build_announce_url(&self, request: &AnnounceRequest) -> Result<String, TrackerError> {
        let mut url = self.announce_url.clone();

        // Add query parameters
        let mut params = Vec::new();

        // Info hash (URL-encoded)
        let info_hash_encoded = Self::url_encode_hash(&request.info_hash);
        params.push(format!("info_hash={}", info_hash_encoded));

        // Peer ID (URL-encoded)
        let peer_id_encoded = Self::url_encode_hash(&request.peer_id);
        params.push(format!("peer_id={}", peer_id_encoded));

        // Port
        params.push(format!("port={}", request.port));

        // Upload/download/left
        params.push(format!("uploaded={}", request.uploaded));
        params.push(format!("downloaded={}", request.downloaded));
        params.push(format!("left={}", request.left));

        // Event (optional)
        if let Some(event) = request.event {
            params.push(format!("event={}", event));
        }

        // IP (optional)
        if let Some(ip) = &request.ip {
            params.push(format!("ip={}", ip));
        }

        // Numwant (optional)
        if let Some(numwant) = request.numwant {
            params.push(format!("numwant={}", numwant));
        }

        // Key (optional)
        if let Some(key) = &request.key {
            params.push(format!("key={}", key));
        }

        // Tracker ID (optional)
        if let Some(trackerid) = &request.trackerid {
            params.push(format!("trackerid={}", urlencoding::encode(trackerid)));
        }

        // Compact format
        params.push(format!("compact={}", if request.compact { 1 } else { 0 }));

        // Join parameters
        if url.contains('?') {
            url.push('&');
        } else {
            url.push('?');
        }
        url.push_str(&params.join("&"));

        Ok(url)
    }

    /// Parse tracker response
    pub fn parse_response(data: &[u8]) -> Result<AnnounceResponse, TrackerError> {
        let root = BencodeParser::parse(data)
            .map_err(|e| TrackerError::InvalidFormat(e.to_string()))?;

        let dict = root.as_dict()
            .ok_or_else(|| TrackerError::InvalidFormat("Response must be a dictionary".to_string()))?;

        // Check for error
        if let Some(error) = dict.get("failure reason") {
            if let Some(msg) = error.as_str() {
                return Err(TrackerError::TrackerError(msg.to_string()));
            }
        }

        // Extract interval
        let interval = dict
            .get("interval")
            .and_then(|v| v.as_int())
            .ok_or_else(|| TrackerError::InvalidFormat("Missing interval".to_string()))? as u32;

        // Extract optional fields
        let min_interval = dict
            .get("min interval")
            .and_then(|v| v.as_int())
            .map(|i| i as u32);

        let trackerid = dict
            .get("tracker id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let complete = dict
            .get("complete")
            .and_then(|v| v.as_int())
            .unwrap_or(0) as u32;

        let incomplete = dict
            .get("incomplete")
            .and_then(|v| v.as_int())
            .unwrap_or(0) as u32;

        // Parse peers
        let mut peers = Vec::new();

        if let Some(peers_value) = dict.get("peers") {
            // Check for compact format (raw bytes)
            if let Some(peers_bytes) = peers_value.as_bytes() {
                peers = Self::parse_compact_peers(peers_bytes);
            } else if let Some(peers_list) = peers_value.as_list() {
                // Dictionary format
                for peer_value in peers_list {
                    if let Some(peer_dict) = peer_value.as_dict() {
                        let ip = peer_dict
                            .get("ip")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let port = peer_dict
                            .get("port")
                            .and_then(|v| v.as_int())
                            .map(|p| p as u16);

                        if let (Some(ip), Some(port)) = (ip, port) {
                            let peer_id = peer_dict
                                .get("peer id")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());

                            peers.push(TrackerPeer { peer_id, ip, port });
                        }
                    }
                }
            }
        }

        Ok(AnnounceResponse {
            interval,
            min_interval,
            trackerid,
            complete,
            incomplete,
            peers,
        })
    }

    /// Parse compact peer list (6 bytes per peer: 4 bytes IP + 2 bytes port)
    fn parse_compact_peers(data: &[u8]) -> Vec<TrackerPeer> {
        let mut peers = Vec::new();

        for chunk in data.chunks(6) {
            if chunk.len() == 6 {
                let ip = format!(
                    "{}.{}.{}.{}",
                    chunk[0], chunk[1], chunk[2], chunk[3]
                );
                let port = u16::from_be_bytes([chunk[4], chunk[5]]);
                peers.push(TrackerPeer {
                    peer_id: None,
                    ip,
                    port,
                });
            }
        }

        peers
    }

    /// URL encode binary data (as % escapes)
    fn url_encode_hash(data: &[u8]) -> String {
        data.iter()
            .map(|&b| format!("%{:02X}", b))
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_event_display() {
        assert_eq!(TrackerEvent::Started.to_string(), "started");
        assert_eq!(TrackerEvent::Completed.to_string(), "completed");
        assert_eq!(TrackerEvent::Stopped.to_string(), "stopped");
    }

    #[test]
    fn test_build_announce_url() {
        let tracker = HttpTracker::new("http://tracker.example.com/announce".to_string());
        let request = AnnounceRequest {
            info_hash: vec![1u8; 20],
            peer_id: vec![2u8; 20],
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: 1024,
            event: Some(TrackerEvent::Started),
            ip: None,
            numwant: Some(30),
            key: None,
            trackerid: None,
            compact: true,
        };

        let url = tracker.build_announce_url(&request).unwrap();
        assert!(url.contains("http://tracker.example.com/announce?"));
        assert!(url.contains("port=6881"));
        assert!(url.contains("event=started"));
        assert!(url.contains("compact=1"));
    }

    #[test]
    fn test_parse_compact_peers() {
        // Create compact peer data: IP 127.0.0.1, Port 6881
        let data = vec![127, 0, 0, 1, 0x1A, 0xE1]; // 0x1AE1 = 6881
        let peers = HttpTracker::parse_compact_peers(&data);
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].ip, "127.0.0.1");
        assert_eq!(peers[0].port, 6881);
    }

    #[test]
    fn test_url_encode_hash() {
        let data = vec![0x12, 0x34, 0x56];
        let encoded = HttpTracker::url_encode_hash(&data);
        assert_eq!(encoded, "%12%34%56");
    }

    #[test]
    fn test_parse_compact_peers_multiple() {
        // Create data for 2 peers
        let mut data = vec![127, 0, 0, 1, 0x1A, 0xE1]; // Peer 1
        data.extend_from_slice(&[192, 168, 1, 1, 0x1A, 0xE1]); // Peer 2
        let peers = HttpTracker::parse_compact_peers(&data);
        assert_eq!(peers.len(), 2);
        assert_eq!(peers[0].ip, "127.0.0.1");
        assert_eq!(peers[1].ip, "192.168.1.1");
    }
}
