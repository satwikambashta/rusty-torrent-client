/// Torrent File Parsing Module
///
/// Handles parsing and validation of .torrent files
/// Extracts metadata including:
/// - File information (name, size, pieces)
/// - Tracker information
/// - Torrent info hash (SHA-1 of info dict)
/// - Piece hashes for verification

use crate::modules::parser::{BencodeParser, BencodeValue};
use serde::{Deserialize, Serialize};
use sha1::{Sha1, Digest};
use std::path::PathBuf;

/// Complete torrent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentMetadata {
    /// Info hash (20 bytes, SHA-1 of info dictionary)
    pub info_hash: Vec<u8>,
    /// Hex-encoded info hash for display
    pub info_hash_hex: String,
    /// Torrent name
    pub name: String,
    /// Total size of files in torrent (bytes)
    pub total_length: u64,
    /// Piece length (bytes)
    pub piece_length: u64,
    /// Number of pieces
    pub pieces_count: u32,
    /// List of piece hashes (20 bytes each)
    pub pieces: Vec<Vec<u8>>,
    /// Files in torrent
    pub files: Vec<FileInfo>,
    /// Announce URL (primary tracker)
    pub announce: String,
    /// Announce list (backup trackers)
    pub announce_list: Vec<Vec<String>>,
    /// Creation date (Unix timestamp, if available)
    pub creation_date: Option<i64>,
    /// Comment (if available)
    pub comment: Option<String>,
}

/// File information within a torrent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// File path components
    pub path: Vec<String>,
    /// File size (bytes)
    pub length: u64,
}

impl FileInfo {
    /// Get full file path
    pub fn full_path(&self) -> PathBuf {
        self.path.iter().collect()
    }

    /// Display path as string
    pub fn display_path(&self) -> String {
        self.path.join("/")
    }
}

/// Torrent parser errors
#[derive(Debug)]
pub enum TorrentParseError {
    /// Invalid bencode format
    Bencode(String),
    /// Missing required field
    MissingField(String),
    /// Invalid field format
    InvalidField(String),
    /// File I/O error
    Io(String),
}

impl std::fmt::Display for TorrentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TorrentParseError::Bencode(msg) => write!(f, "Bencode error: {}", msg),
            TorrentParseError::MissingField(field) => write!(f, "Missing field: {}", field),
            TorrentParseError::InvalidField(msg) => write!(f, "Invalid field: {}", msg),
            TorrentParseError::Io(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl std::error::Error for TorrentParseError {}

impl From<std::io::Error> for TorrentParseError {
    fn from(e: std::io::Error) -> Self {
        TorrentParseError::Io(e.to_string())
    }
}

/// Torrent file parser
pub struct TorrentParser;

impl TorrentParser {
    /// Parse a torrent file from bytes
    pub fn parse(data: &[u8]) -> Result<TorrentMetadata, TorrentParseError> {
        let root = BencodeParser::parse(data)
            .map_err(|e| TorrentParseError::Bencode(e.to_string()))?;

        let root_dict = root.as_dict()
            .ok_or_else(|| TorrentParseError::InvalidField("Root must be a dictionary".to_string()))?;

        // Extract announce (required)
        let announce = root_dict
            .get("announce")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TorrentParseError::MissingField("announce".to_string()))?
            .to_string();

        // Extract announce-list (optional, for redundancy)
        let announce_list = if let Some(list_value) = root_dict.get("announce-list") {
            if let Some(list) = list_value.as_list() {
                list.iter()
                    .filter_map(|tier| {
                        tier.as_list().map(|tier_list| {
                            tier_list
                                .iter()
                                .filter_map(|url| url.as_str().map(|s| s.to_string()))
                                .collect::<Vec<_>>()
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        // Extract optional fields
        let creation_date = root_dict
            .get("creation date")
            .and_then(|v| v.as_int());

        let comment = root_dict
            .get("comment")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract info dictionary (required)
        let info_dict = root_dict
            .get("info")
            .and_then(|v| v.as_dict())
            .ok_or_else(|| TorrentParseError::MissingField("info".to_string()))?;

        // Calculate info hash (SHA-1 of bencode-encoded info dict)
        let info_hash = Self::calculate_info_hash(info_dict)?;
        let info_hash_hex = Self::bytes_to_hex(&info_hash);

        // Extract torrent name
        let name = info_dict
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TorrentParseError::MissingField("info.name".to_string()))?
            .to_string();

        // Extract piece length
        let piece_length = info_dict
            .get("piece length")
            .and_then(|v| v.as_int())
            .ok_or_else(|| TorrentParseError::MissingField("info.piece length".to_string()))?
            as u64;

        if piece_length == 0 {
            return Err(TorrentParseError::InvalidField(
                "piece length must be > 0".to_string(),
            ));
        }

        // Parse pieces hashes
        let pieces_raw = info_dict
            .get("pieces")
            .and_then(|v| v.as_bytes())
            .ok_or_else(|| TorrentParseError::MissingField("info.pieces".to_string()))?;

        if pieces_raw.len() % 20 != 0 {
            return Err(TorrentParseError::InvalidField(
                "pieces hash length must be multiple of 20".to_string(),
            ));
        }

        let pieces = pieces_raw
            .chunks(20)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>();

        let pieces_count = pieces.len() as u32;

        // Parse files
        let (files, total_length) = if let Some(files_value) = info_dict.get("files") {
            if let Some(files_list) = files_value.as_list() {
                // Multi-file torrent
                let mut files = Vec::new();
                let mut total = 0u64;

                for file_dict_value in files_list {
                    let file_dict = file_dict_value.as_dict()
                        .ok_or_else(|| TorrentParseError::InvalidField(
                            "files entry must be a dict".to_string(),
                        ))?;

                    let length = file_dict
                        .get("length")
                        .and_then(|v| v.as_int())
                        .ok_or_else(|| TorrentParseError::MissingField(
                            "file length".to_string(),
                        ))? as u64;

                    let path_list = file_dict
                        .get("path")
                        .and_then(|v| v.as_list())
                        .ok_or_else(|| TorrentParseError::MissingField(
                            "file path".to_string(),
                        ))?;

                    let path = path_list
                        .iter()
                        .filter_map(|p| p.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>();

                    if path.is_empty() {
                        return Err(TorrentParseError::InvalidField(
                            "file path cannot be empty".to_string(),
                        ));
                    }

                    files.push(FileInfo { path, length });
                    total += length;
                }

                (files, total)
            } else {
                return Err(TorrentParseError::InvalidField(
                    "files must be a list".to_string(),
                ));
            }
        } else {
            // Single-file torrent
            let length = info_dict
                .get("length")
                .and_then(|v| v.as_int())
                .ok_or_else(|| TorrentParseError::MissingField(
                    "info.length (single file)".to_string(),
                ))? as u64;

            let file = FileInfo {
                path: vec![name.clone()],
                length,
            };

            (vec![file], length)
        };

        // Validate pieces count matches file size
        let expected_pieces = (total_length + piece_length - 1) / piece_length;
        if pieces_count as u64 != expected_pieces {
            tracing::warn!(
                "Piece count mismatch: expected {}, got {}",
                expected_pieces,
                pieces_count
            );
        }

        Ok(TorrentMetadata {
            info_hash,
            info_hash_hex,
            name,
            total_length,
            piece_length,
            pieces_count,
            pieces,
            files,
            announce,
            announce_list,
            creation_date,
            comment,
        })
    }

    /// Parse a torrent file from filesystem
    pub fn parse_file(path: &PathBuf) -> Result<TorrentMetadata, TorrentParseError> {
        let data = std::fs::read(path)?;
        Self::parse(&data)
    }

    /// Calculate info hash (SHA-1 of bencoded info dictionary)
    fn calculate_info_hash(info_dict: &std::collections::BTreeMap<String, BencodeValue>) -> Result<Vec<u8>, TorrentParseError> {
        let encoded = BencodeParser::encode(&BencodeValue::Dict(info_dict.clone()));
        let mut hasher = Sha1::new();
        hasher.update(&encoded);
        Ok(hasher.finalize().to_vec())
    }

    /// Convert bytes to hex string
    fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::parser::BencodeValue;
    use bytes::Bytes;

    fn create_simple_torrent() -> Vec<u8> {
        // Manually create a simple torrent structure
        // d8:announce<url>4:infod6:lengthi1000e4:name5:test_12:pll20:xxxxxxxxxxxxxxxxxxxxxxee
        let mut dict = std::collections::BTreeMap::new();
        dict.insert(
            "announce".to_string(),
            BencodeValue::String(Bytes::from_static(b"http://tracker.example.com/announce")),
        );

        let mut info = std::collections::BTreeMap::new();
        info.insert(
            "name".to_string(),
            BencodeValue::String(Bytes::from_static(b"test_file")),
        );
        info.insert(
            "length".to_string(),
            BencodeValue::Integer(1024),
        );
        info.insert(
            "piece length".to_string(),
            BencodeValue::Integer(16384),
        );

        // Create 20 bytes of zeros for piece hash
        let piece_hash = vec![0u8; 20];
        info.insert(
            "pieces".to_string(),
            BencodeValue::String(Bytes::from(piece_hash)),
        );

        dict.insert("info".to_string(), BencodeValue::Dict(info));

        BencodeParser::encode(&BencodeValue::Dict(dict))
    }

    #[test]
    fn test_parse_simple_torrent() {
        let data = create_simple_torrent();
        let result = TorrentParser::parse(&data);
        assert!(result.is_ok());

        let metadata = result.unwrap();
        assert_eq!(metadata.name, "test_file");
        assert_eq!(metadata.total_length, 1024);
        assert_eq!(metadata.piece_length, 16384);
        assert_eq!(metadata.announce, "http://tracker.example.com/announce");
    }

    #[test]
    fn test_info_hash_calculation() {
        let data = create_simple_torrent();
        let metadata = TorrentParser::parse(&data).unwrap();
        assert_eq!(metadata.info_hash.len(), 20);
        assert!(!metadata.info_hash_hex.is_empty());
    }

    #[test]
    fn test_invalid_bencode() {
        let result = TorrentParser::parse(b"invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_info() {
        let dict = std::collections::BTreeMap::new();
        let value = BencodeValue::Dict(dict);
        let data = BencodeParser::encode(&value);
        let result = TorrentParser::parse(&data);
        assert!(result.is_err());
    }
}
