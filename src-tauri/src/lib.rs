// Rusty Torrents - Backend
mod commands;
mod modules;

use modules::config::Config;
use modules::logging;
use crate::commands::peer_wire::PeerWireState;
use crate::commands::seeding::SeederState;

// Application entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    let config = Config::new();
    if let Err(e) = logging::init_logging(&config.log_dir, config.verbose_logging) {
        eprintln!("Failed to initialize logging: {}", e);
    }

    // Ensure directories exist
    if let Err(e) = config.ensure_directories() {
        tracing::error!("Failed to create directories: {}", e);
    }

    tracing::info!("Starting Rusty Torrents v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(PeerWireState::new())
        .manage(SeederState::new())
        .invoke_handler(tauri::generate_handler![
            // Test commands
            commands::test_connection,
            commands::get_server_info,
            // Torrent commands
            commands::get_torrents,
            commands::add_torrent,
            commands::start_torrent,
            commands::pause_torrent,
            commands::remove_torrent,
            // Search & Scan commands
            commands::search_torrents,
            commands::scan_folder,
            commands::get_config,
            commands::update_config,
            // Peer discovery commands
            commands::discover_peers_dht,
            commands::announce_to_tracker,
            commands::add_discovered_peers,
            commands::get_peer_pool_status,
            commands::connect_to_peers,
            // Download commands (Phase 2-4)
            commands::parse_torrent_file,
            commands::start_download,
            commands::get_download_progress,
            commands::pause_download,
            commands::resume_download,
            commands::cancel_download,
            // Peer wire protocol commands (Phase 5)
            commands::connect_to_peer,
            commands::disconnect_from_peer,
            commands::choke_peer,
            commands::unchoke_peer,
            commands::express_interest,
            commands::express_not_interested,
            commands::request_piece_block,
            commands::send_piece_block,
            commands::broadcast_have_piece,
            commands::receive_peer_messages,
            commands::get_peer_connection_stats,
            commands::peer_has_piece,
            commands::get_peers_with_piece,
            // Seeding commands (Phase 6)
            commands::register_seeding_peer,
            commands::unregister_seeding_peer,
            commands::seeding_peer_interested,
            commands::request_block_upload,
            commands::run_choking_algorithm,
            commands::update_seeding_config,
            commands::get_seeding_stats,
            commands::get_seeding_peers,
            commands::cleanup_idle_seeding_peers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



