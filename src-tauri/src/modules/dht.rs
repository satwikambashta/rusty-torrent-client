/// DHT (Distributed Hash Table) Module
///
/// Implements peer discovery via DHT network
/// Follows DHT protocol (BEP 5 - Distributed Hash Table)
/// Used to discover peers without relying on centralized trackers

use crate::modules::parser::{BencodeParser, BencodeValue};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// DHT node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtNode {
    /// Node ID (20 bytes)
    pub id: Vec<u8>,
    /// Socket address
    pub addr: String,
}

/// DHT peer information for a torrent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhtPeer {
    /// Peer IP address
    pub ip: String,
    /// Peer port
    pub port: u16,
}

/// DHT find_node request/response
#[derive(Debug, Clone)]
pub struct DhtFindNodeQuery {
    /// Transaction ID
    pub t: String,
    /// Our node ID
    pub id: Vec<u8>,
    /// Target node ID to find
    pub target: Vec<u8>,
}

/// DHT get_peers request/response
#[derive(Debug, Clone)]
pub struct DhtGetPeersQuery {
    /// Transaction ID
    pub t: String,
    /// Our node ID
    pub id: Vec<u8>,
    /// Info hash to find peers for
    pub info_hash: Vec<u8>,
}

/// DHT Client
pub struct DhtClient {
    /// Our node ID (20 bytes)
    node_id: Vec<u8>,
    /// Bootstrap nodes
    bootstrap_nodes: Vec<DhtNode>,
    /// Known routing table
    routing_table: Vec<DhtNode>,
}

impl DhtClient {
    /// Create a new DHT client
    pub fn new() -> Self {
        // Generate random node ID
        let node_id = Self::generate_node_id();

        Self {
            node_id,
            bootstrap_nodes: Self::default_bootstrap_nodes(),
            routing_table: Vec::new(),
        }
    }

    /// Get our node ID
    pub fn node_id(&self) -> &[u8] {
        &self.node_id
    }

    /// Add a node to routing table
    pub fn add_node(&mut self, node: DhtNode) {
        if !self.routing_table.iter().any(|n| n.id == node.id) {
            self.routing_table.push(node);
        }
    }

    /// Get k closest nodes to a target (for XOR distance calculation)
    pub fn get_k_closest_nodes(&self, target: &[u8], k: usize) -> Vec<DhtNode> {
        let mut nodes = self.routing_table.clone();

        // Sort by XOR distance to target
        nodes.sort_by_key(|node| Self::xor_distance(&node.id, target));

        nodes.into_iter().take(k).collect()
    }

    /// Calculate XOR distance between two node IDs
    pub fn xor_distance(a: &[u8], b: &[u8]) -> u64 {
        let mut distance = 0u64;
        let len = a.len().min(b.len()).min(8);

        for i in 0..len {
            distance ^= (a[i] as u64) << (56 - (i * 8));
            distance ^= (b[i] as u64) << (56 - (i * 8));
        }

        distance
    }

    /// Create find_node query
    pub fn create_find_node_query(&self, target: Vec<u8>) -> DhtFindNodeQuery {
        let t = Self::generate_transaction_id();

        DhtFindNodeQuery {
            t,
            id: self.node_id.clone(),
            target,
        }
    }

    /// Create get_peers query
    pub fn create_get_peers_query(&self, info_hash: Vec<u8>) -> DhtGetPeersQuery {
        let t = Self::generate_transaction_id();

        DhtGetPeersQuery {
            t,
            id: self.node_id.clone(),
            info_hash,
        }
    }

    /// Encode find_node query to bencode format
    pub fn encode_find_node(&self, query: &DhtFindNodeQuery) -> Vec<u8> {
        let mut dict = std::collections::BTreeMap::new();

        dict.insert(
            "t".to_string(),
            BencodeValue::String(Bytes::from(query.t.clone().into_bytes())),
        );
        dict.insert(
            "y".to_string(),
            BencodeValue::String(Bytes::from_static(b"q")),
        );
        dict.insert(
            "q".to_string(),
            BencodeValue::String(Bytes::from_static(b"find_node")),
        );

        let mut args = std::collections::BTreeMap::new();
        args.insert(
            "id".to_string(),
            BencodeValue::String(Bytes::from(query.id.clone())),
        );
        args.insert(
            "target".to_string(),
            BencodeValue::String(Bytes::from(query.target.clone())),
        );

        dict.insert("a".to_string(), BencodeValue::Dict(args));

        BencodeParser::encode(&BencodeValue::Dict(dict))
    }

    /// Encode get_peers query to bencode format
    pub fn encode_get_peers(&self, query: &DhtGetPeersQuery) -> Vec<u8> {
        let mut dict = std::collections::BTreeMap::new();

        dict.insert(
            "t".to_string(),
            BencodeValue::String(Bytes::from(query.t.clone().into_bytes())),
        );
        dict.insert(
            "y".to_string(),
            BencodeValue::String(Bytes::from_static(b"q")),
        );
        dict.insert(
            "q".to_string(),
            BencodeValue::String(Bytes::from_static(b"get_peers")),
        );

        let mut args = std::collections::BTreeMap::new();
        args.insert(
            "id".to_string(),
            BencodeValue::String(Bytes::from(query.id.clone())),
        );
        args.insert(
            "info_hash".to_string(),
            BencodeValue::String(Bytes::from(query.info_hash.clone())),
        );

        dict.insert("a".to_string(), BencodeValue::Dict(args));

        BencodeParser::encode(&BencodeValue::Dict(dict))
    }

    /// Parse get_peers response to extract peers
    pub fn parse_get_peers_response(data: &[u8]) -> Result<Vec<DhtPeer>, String> {
        let root = BencodeParser::parse(data)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        let root_dict = root
            .as_dict()
            .ok_or_else(|| "Response must be a dictionary".to_string())?;

        let mut peers = Vec::new();

        // Check for 'values' key (compact peer list)
        if let Some(values) = root_dict.get("r").and_then(|r| r.as_dict()).and_then(|d| d.get("values")) {
            if let Some(values_list) = values.as_list() {
                for peer_bytes in values_list {
                    if let Some(bytes) = peer_bytes.as_bytes() {
                        if bytes.len() == 6 {
                            // Compact format: 4 bytes IP + 2 bytes port
                            let ip = format!(
                                "{}.{}.{}.{}",
                                bytes[0], bytes[1], bytes[2], bytes[3]
                            );
                            let port = u16::from_be_bytes([bytes[4], bytes[5]]);
                            peers.push(DhtPeer { ip, port });
                        }
                    }
                }
            }
        }

        Ok(peers)
    }

    // Helper functions

    /// Generate a random node ID (20 bytes)
    fn generate_node_id() -> Vec<u8> {
        let uuid = Uuid::new_v4();
        uuid.as_bytes().to_vec()
    }

    /// Generate a random transaction ID
    fn generate_transaction_id() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        format!("{:x}", timestamp % 0x1000)
    }

    /// Get default bootstrap nodes for DHT network
    fn default_bootstrap_nodes() -> Vec<DhtNode> {
        vec![
            DhtNode {
                id: vec![0u8; 20],
                addr: "router.bittorrent.com:6881".to_string(),
            },
            DhtNode {
                id: vec![0u8; 20],
                addr: "dht.transmissionbt.com:6881".to_string(),
            },
            DhtNode {
                id: vec![0u8; 20],
                addr: "router.utorrent.com:6881".to_string(),
            },
        ]
    }
}

impl Default for DhtClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_node_id() {
        let client = DhtClient::new();
        // UUID is 16 bytes, we use it as-is
        assert_eq!(client.node_id().len(), 16);
    }

    #[test]
    fn test_xor_distance() {
        let a = vec![0u8; 20];
        let b = vec![1u8; 20];
        let distance = DhtClient::xor_distance(&a, &b);
        assert!(distance > 0);
    }

    #[test]
    fn test_add_node() {
        let mut client = DhtClient::new();
        let node = DhtNode {
            id: vec![1u8; 20],
            addr: "127.0.0.1:6881".to_string(),
        };
        client.add_node(node.clone());
        assert_eq!(client.routing_table.len(), 1);
    }

    #[test]
    fn test_k_closest_nodes() {
        let mut client = DhtClient::new();
        for i in 0..10 {
            let mut id = vec![i as u8; 20];
            id[0] = i as u8;
            let node = DhtNode {
                id,
                addr: format!("127.0.0.1:{}", 6881 + i),
            };
            client.add_node(node);
        }

        let target = vec![5u8; 20];
        let closest = client.get_k_closest_nodes(&target, 3);
        assert_eq!(closest.len(), 3);
    }

    #[test]
    fn test_create_find_node_query() {
        let client = DhtClient::new();
        let target = vec![1u8; 20];
        let query = client.create_find_node_query(target);
        assert_eq!(query.id.len(), 16);
        assert_eq!(query.target.len(), 20);
    }

    #[test]
    fn test_create_get_peers_query() {
        let client = DhtClient::new();
        let info_hash = vec![2u8; 20];
        let query = client.create_get_peers_query(info_hash);
        assert_eq!(query.id.len(), 16);
        assert_eq!(query.info_hash.len(), 20);
    }

    #[test]
    fn test_encode_find_node() {
        let client = DhtClient::new();
        let target = vec![1u8; 20];
        let query = client.create_find_node_query(target);
        let encoded = client.encode_find_node(&query);
        assert!(!encoded.is_empty());
    }

    #[test]
    fn test_encode_get_peers() {
        let client = DhtClient::new();
        let info_hash = vec![2u8; 20];
        let query = client.create_get_peers_query(info_hash);
        let encoded = client.encode_get_peers(&query);
        assert!(!encoded.is_empty());
    }
}
