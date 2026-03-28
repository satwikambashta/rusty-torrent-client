// Rusty Torrents - Backend
mod commands;
mod modules;

use modules::config::Config;
use modules::logging;

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
            commands::get_seeding_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}



