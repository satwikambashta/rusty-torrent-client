use serde::{Deserialize, Serialize};

/// Response from the backend test
#[derive(Debug, Serialize, Deserialize)]
pub struct TestResponse {
    pub status: String,
    pub message: String,
    pub timestamp: String,
    pub backend_version: String,
}

/// Test endpoint to verify backend-frontend connectivity
#[tauri::command]
pub fn test_connection() -> TestResponse {
    TestResponse {
        status: "success".to_string(),
        message: "Backend is connected and responding!".to_string(),
        timestamp: chrono::Local::now().to_rfc3339(),
        backend_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Get server information
#[tauri::command]
pub fn get_server_info() -> TestResponse {
    TestResponse {
        status: "success".to_string(),
        message: "Rusty Torrents Server v0.1.0".to_string(),
        timestamp: chrono::Local::now().to_rfc3339(),
        backend_version: env!("CARGO_PKG_VERSION").to_string(),
    }
}
