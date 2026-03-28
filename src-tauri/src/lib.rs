// Rusty Torrents - Backend
mod commands;
mod modules;

// Application entry point
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


