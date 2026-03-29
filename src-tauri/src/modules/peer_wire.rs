use std::collections::HashMap;
use std::io::{Read, Write};
use tokio::net::TcpStream;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio::time::timeout;
use serde::{Deserialize, Serialize};
use crate::modules::peer::{Peer, PeerState};
use std::default::Default;

/// Serializable peer connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnectionStats {
    pub addr: String,
    pub am_choking: bool,
    pub am_interested: bool,
    pub peer_choking: bool,
    pub peer_interested: bool,
    pub piece_count: u32,
    pub download_rate: f64,
    pub upload_rate: f64,
}

/// BitTorrent peer wire protocol messages
#[derive(Debug, Clone)]
pub enum PeerMessage {
    KeepAlive,
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have { piece_index: u32 },
    Bitfield { bitfield: Vec<u8> },
    Request { index: u32, begin: u32, length: u32 },
    Piece { index: u32, begin: u32, block: Vec<u8> },
    Cancel { index: u32, begin: u32, length: u32 },
    Port { listen_port: u16 },
}

/// Peer connection state
#[derive(Debug)]
pub struct PeerConnection {
    pub peer: Peer,
    pub stream: Option<TcpStream>,
    pub am_choking: bool,
    pub am_interested: bool,
    pub peer_choking: bool,
    pub peer_interested: bool,
    pub bitfield: Vec<u8>,
    pub last_message_time: Instant,
    pub download_rate: f64,
    pub upload_rate: f64,
}

impl PeerConnection {
    pub fn new(peer: Peer) -> Self {
        Self {
            peer,
            stream: None,
            am_choking: true,
            am_interested: false,
            peer_choking: true,
            peer_interested: false,
            bitfield: Vec::new(),
            last_message_time: Instant::now(),
            download_rate: 0.0,
            upload_rate: 0.0,
        }
    }

    /// Perform BitTorrent handshake
    pub async fn handshake(&mut self, info_hash: &[u8; 20], peer_id: &[u8; 20]) -> Result<(), Box<dyn std::error::Error>> {
        // Parse peer address
        let addr_parts: Vec<&str> = self.peer.addr.split(':').collect();
        if addr_parts.len() != 2 {
            return Err("Invalid peer address format".into());
        }
        let ip = addr_parts[0].parse()?;
        let port: u16 = addr_parts[1].parse()?;
        let addr = std::net::SocketAddr::new(ip, port);

        let mut stream = timeout(Duration::from_secs(10), TcpStream::connect(addr)).await??;

        // Send handshake
        let mut handshake = Vec::new();
        handshake.push(19); // Protocol string length
        handshake.extend_from_slice(b"BitTorrent protocol");
        handshake.extend_from_slice(&[0u8; 8]); // Reserved bytes
        handshake.extend_from_slice(info_hash);
        handshake.extend_from_slice(peer_id);

        stream.write_all(&handshake).await?;

        // Read response handshake
        let mut response = [0u8; 68];
        stream.read_exact(&mut response).await?;

        self.stream = Some(stream);

        // Verify protocol
        if response[0] != 19 {
            return Err("Invalid protocol length".into());
        }
        if &response[1..20] != b"BitTorrent protocol" {
            return Err("Invalid protocol string".into());
        }

        // Verify info hash matches
        if &response[28..48] != info_hash {
            return Err("Info hash mismatch".into());
        }

        self.last_message_time = Instant::now();
        Ok(())
    }

    /// Send a message to the peer
    pub async fn send_message(&mut self, message: PeerMessage) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut stream) = self.stream {
            let payload = match message {
                PeerMessage::KeepAlive => vec![0u8; 4],
                PeerMessage::Choke => {
                    let mut msg = vec![0u8; 4];
                    msg[3] = 1; // Length
                    msg.push(0); // ID
                    msg
                },
                PeerMessage::Unchoke => {
                    let mut msg = vec![0u8; 4];
                    msg[3] = 1;
                    msg.push(1);
                    msg
                },
                PeerMessage::Interested => {
                    let mut msg = vec![0u8; 4];
                    msg[3] = 1;
                    msg.push(2);
                    msg
                },
                PeerMessage::NotInterested => {
                    let mut msg = vec![0u8; 4];
                    msg[3] = 1;
                    msg.push(3);
                    msg
                },
                PeerMessage::Have { piece_index } => {
                    let mut msg = vec![0u8; 4];
                    msg[3] = 5;
                    msg.push(4);
                    msg.extend_from_slice(&piece_index.to_be_bytes());
                    msg
                },
                PeerMessage::Bitfield { bitfield } => {
                    let mut msg = vec![0u8; 4];
                    let len = 1 + bitfield.len() as u32;
                    msg[0..4].copy_from_slice(&len.to_be_bytes());
                    msg.push(5);
                    msg.extend_from_slice(&bitfield);
                    msg
                },
                PeerMessage::Request { index, begin, length } => {
                    let mut msg = vec![0u8; 4];
                    msg[3] = 13;
                    msg.push(6);
                    msg.extend_from_slice(&index.to_be_bytes());
                    msg.extend_from_slice(&begin.to_be_bytes());
                    msg.extend_from_slice(&length.to_be_bytes());
                    msg
                },
                PeerMessage::Piece { index, begin, block } => {
                    let mut msg = vec![0u8; 4];
                    let len = 9 + block.len() as u32;
                    msg[0..4].copy_from_slice(&len.to_be_bytes());
                    msg.push(7);
                    msg.extend_from_slice(&index.to_be_bytes());
                    msg.extend_from_slice(&begin.to_be_bytes());
                    msg.extend_from_slice(&block);
                    msg
                },
                PeerMessage::Cancel { index, begin, length } => {
                    let mut msg = vec![0u8; 4];
                    msg[3] = 13;
                    msg.push(8);
                    msg.extend_from_slice(&index.to_be_bytes());
                    msg.extend_from_slice(&begin.to_be_bytes());
                    msg.extend_from_slice(&length.to_be_bytes());
                    msg
                },
                PeerMessage::Port { listen_port } => {
                    let mut msg = vec![0u8; 4];
                    msg[3] = 3;
                    msg.push(9);
                    msg.extend_from_slice(&listen_port.to_be_bytes());
                    msg
                },
            };

            stream.write_all(&payload).await?;
            self.last_message_time = Instant::now();
        }
        Ok(())
    }

    /// Receive and parse a message from the peer
    pub async fn receive_message(&mut self) -> Result<Option<PeerMessage>, Box<dyn std::error::Error>> {
        if let Some(ref mut stream) = self.stream {
            // Read message length (4 bytes)
            let mut length_buf = [0u8; 4];
            stream.read_exact(&mut length_buf).await?;
            let length = u32::from_be_bytes(length_buf);

            if length == 0 {
                return Ok(Some(PeerMessage::KeepAlive));
            }

            // Read message ID
            let mut id_buf = [0u8; 1];
            stream.read_exact(&mut id_buf).await?;
            let id = id_buf[0];

            // Read payload based on message type
            let message = match id {
                0 => {
                    self.peer_choking = true;
                    Some(PeerMessage::Choke)
                },
                1 => {
                    self.peer_choking = false;
                    Some(PeerMessage::Unchoke)
                },
                2 => {
                    self.peer_interested = true;
                    Some(PeerMessage::Interested)
                },
                3 => {
                    self.peer_interested = false;
                    Some(PeerMessage::NotInterested)
                },
                4 => {
                    let mut piece_buf = [0u8; 4];
                    stream.read_exact(&mut piece_buf).await?;
                    let piece_index = u32::from_be_bytes(piece_buf);
                    Some(PeerMessage::Have { piece_index })
                },
                5 => {
                    let bitfield_len = length - 1;
                    let mut bitfield = vec![0u8; bitfield_len as usize];
                    stream.read_exact(&mut bitfield).await?;
                    self.bitfield = bitfield.clone();
                    Some(PeerMessage::Bitfield { bitfield })
                },
                6 => {
                    let mut request_buf = [0u8; 12];
                    stream.read_exact(&mut request_buf).await?;
                    let index = u32::from_be_bytes(request_buf[0..4].try_into()?);
                    let begin = u32::from_be_bytes(request_buf[4..8].try_into()?);
                    let length = u32::from_be_bytes(request_buf[8..12].try_into()?);
                    Some(PeerMessage::Request { index, begin, length })
                },
                7 => {
                    let mut piece_header = [0u8; 8];
                    stream.read_exact(&mut piece_header).await?;
                    let index = u32::from_be_bytes(piece_header[0..4].try_into()?);
                    let begin = u32::from_be_bytes(piece_header[4..8].try_into()?);
                    let block_len = length - 9;
                    let mut block = vec![0u8; block_len as usize];
                    stream.read_exact(&mut block).await?;
                    Some(PeerMessage::Piece { index, begin, block })
                },
                8 => {
                    let mut cancel_buf = [0u8; 12];
                    stream.read_exact(&mut cancel_buf).await?;
                    let index = u32::from_be_bytes(cancel_buf[0..4].try_into()?);
                    let begin = u32::from_be_bytes(cancel_buf[4..8].try_into()?);
                    let length = u32::from_be_bytes(cancel_buf[8..12].try_into()?);
                    Some(PeerMessage::Cancel { index, begin, length })
                },
                9 => {
                    let mut port_buf = [0u8; 2];
                    stream.read_exact(&mut port_buf).await?;
                    let listen_port = u16::from_be_bytes(port_buf);
                    Some(PeerMessage::Port { listen_port })
                },
                _ => {
                    // Unknown message, skip payload
                    let mut skip_buf = vec![0u8; (length - 1) as usize];
                    stream.read_exact(&mut skip_buf).await?;
                    None
                }
            };

            self.last_message_time = Instant::now();
            Ok(message)
        } else {
            Ok(None)
        }
    }

    /// Check if peer has a specific piece
    pub fn has_piece(&self, piece_index: u32) -> bool {
        if self.bitfield.is_empty() {
            return false;
        }
        let byte_index = (piece_index / 8) as usize;
        let bit_index = (piece_index % 8) as u8;
        if byte_index >= self.bitfield.len() {
            return false;
        }
        (self.bitfield[byte_index] & (1 << (7 - bit_index))) != 0
    }

    /// Get the number of pieces this peer has
    pub fn piece_count(&self) -> u32 {
        if self.bitfield.is_empty() {
            return 0;
        }
        self.bitfield.iter().map(|&byte| byte.count_ones()).sum()
    }
}

/// Peer wire protocol manager
#[derive(Clone)]
pub struct PeerWireProtocol {
    connections: Arc<Mutex<HashMap<String, PeerConnection>>>,
    max_connections: usize,
}

impl PeerWireProtocol {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            max_connections,
        }
    }

    /// Connect to a peer and perform handshake
    pub async fn connect_peer(&self, peer: Peer, info_hash: &[u8; 20], peer_id: &[u8; 20]) -> Result<String, Box<dyn std::error::Error>> {
        let mut connections = self.connections.lock().await;

        if connections.len() >= self.max_connections {
            return Err("Max connections reached".into());
        }

        let peer_key = format!("{}", peer.addr);
        if connections.contains_key(&peer_key) {
            return Err("Already connected to peer".into());
        }

        let mut connection = PeerConnection::new(peer);
        connection.handshake(info_hash, peer_id).await?;

        // Send bitfield if we have any pieces
        // For now, send empty bitfield (no pieces downloaded yet)
        connection.send_message(PeerMessage::Bitfield { bitfield: vec![] }).await?;

        connections.insert(peer_key.clone(), connection);
        Ok(peer_key)
    }

    /// Disconnect from a peer
    pub async fn disconnect_peer(&self, peer_key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut connections = self.connections.lock().await;
        connections.remove(peer_key);
        Ok(())
    }

    /// Send a message to a specific peer
    pub async fn send_to_peer(&self, peer_key: &str, message: PeerMessage) -> Result<(), Box<dyn std::error::Error>> {
        let mut connections = self.connections.lock().await;
        if let Some(connection) = connections.get_mut(peer_key) {
            connection.send_message(message).await?;
        }
        Ok(())
    }

    /// Receive messages from all connected peers
    pub async fn receive_messages(&self) -> Result<Vec<(String, PeerMessage)>, Box<dyn std::error::Error>> {
        let mut connections = self.connections.lock().await;
        let mut messages = Vec::new();

        for (peer_key, connection) in connections.iter_mut() {
            if let Ok(Some(message)) = connection.receive_message().await {
                messages.push((peer_key.clone(), message));
            }
        }

        Ok(messages)
    }

    /// Request a block from a peer
    pub async fn request_block(&self, peer_key: &str, piece_index: u32, begin: u32, length: u32) -> Result<(), Box<dyn std::error::Error>> {
        let message = PeerMessage::Request { index: piece_index, begin, length };
        self.send_to_peer(peer_key, message).await
    }

    /// Send a piece block to a peer
    pub async fn send_block(&self, peer_key: &str, piece_index: u32, begin: u32, block: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
        let message = PeerMessage::Piece { index: piece_index, begin, block };
        self.send_to_peer(peer_key, message).await
    }

    /// Notify peers that we have a new piece
    pub async fn broadcast_have(&self, piece_index: u32) -> Result<(), Box<dyn std::error::Error>> {
        let message = PeerMessage::Have { piece_index };
        let peer_keys: Vec<String> = {
            let connections = self.connections.lock().await;
            connections.keys().cloned().collect()
        };

        for peer_key in peer_keys {
            let msg = message.clone();
            let protocol = Arc::new(self.clone());
            tokio::spawn(async move {
                let _ = protocol.send_to_peer(&peer_key, msg).await;
            });
        }

        Ok(())
    }

    /// Get connection statistics
    pub async fn get_stats(&self) -> HashMap<String, PeerConnectionStats> {
        let connections = self.connections.lock().await;
        let mut stats = HashMap::new();

        for (key, connection) in connections.iter() {
            let stat = PeerConnectionStats {
                addr: connection.peer.addr.clone(),
                am_choking: connection.am_choking,
                am_interested: connection.am_interested,
                peer_choking: connection.peer_choking,
                peer_interested: connection.peer_interested,
                piece_count: connection.piece_count(),
                download_rate: connection.download_rate,
                upload_rate: connection.upload_rate,
            };
            stats.insert(key.clone(), stat);
        }

        stats
    }

    /// Check if a peer has a specific piece
    pub async fn peer_has_piece(&self, peer_key: &str, piece_index: u32) -> bool {
        let connections = self.connections.lock().await;
        if let Some(connection) = connections.get(peer_key) {
            connection.has_piece(piece_index)
        } else {
            false
        }
    }

    /// Get list of peers that have a specific piece
    pub async fn get_peers_with_piece(&self, piece_index: u32) -> Vec<String> {
        let connections = self.connections.lock().await;
        connections.iter()
            .filter(|(_, conn)| conn.has_piece(piece_index))
            .map(|(key, _)| key.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_peer_connection_creation() {
        let peer = Peer {
            addr: "127.0.0.1:6881".to_string(),
            peer_id: None,
            state: PeerState::Disconnected,
            upload_speed: 0,
            download_speed: 0,
            have_pieces: Vec::new(),
            interested: false,
            connected_at: None,
            stats: Default::default(),
        };

        let connection = PeerConnection::new(peer.clone());
        assert_eq!(connection.peer.addr, peer.addr);
        assert!(connection.am_choking);
        assert!(!connection.am_interested);
        assert!(connection.peer_choking);
        assert!(!connection.peer_interested);
        assert!(connection.bitfield.is_empty());
    }

    #[test]
    fn test_has_piece() {
        let peer = Peer {
            addr: "127.0.0.1:6881".to_string(),
            peer_id: None,
            state: PeerState::Disconnected,
            upload_speed: 0,
            download_speed: 0,
            have_pieces: Vec::new(),
            interested: false,
            connected_at: None,
            stats: Default::default(),
        };

        let mut connection = PeerConnection::new(peer);
        connection.bitfield = vec![0b10100000]; // Pieces 0 and 2 are available

        assert!(connection.has_piece(0));
        assert!(!connection.has_piece(1));
        assert!(connection.has_piece(2));
        assert!(!connection.has_piece(8)); // Out of bounds
    }

    #[test]
    fn test_piece_count() {
        let peer = Peer {
            addr: "127.0.0.1:6881".to_string(),
            peer_id: None,
            state: PeerState::Disconnected,
            upload_speed: 0,
            download_speed: 0,
            have_pieces: Vec::new(),
            interested: false,
            connected_at: None,
            stats: Default::default(),
        };

        let mut connection = PeerConnection::new(peer);
        connection.bitfield = vec![0b10100000, 0b00001100]; // 4 pieces available

        assert_eq!(connection.piece_count(), 4);
    }
}