// Core modules for the Rusty Torrents
pub mod config;
pub mod torrent;
pub mod logging;
pub mod search;
pub mod scanner;
pub mod web_server;

// Phase 2: Torrent file parsing
pub mod parser;
pub mod torrent_parser;

// Phase 3: DHT and peer discovery
pub mod dht;
pub mod tracker;
pub mod peer;

// Phase 4: Download engine
pub mod download;
pub mod pieces;

// Phase 5: Peer wire protocol
pub mod peer_wire;

// Phase 6: Seeding & Upload
pub mod seeder;
