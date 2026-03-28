use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

use crate::commands::TorrentInfo;
use crate::modules::logging::SeedingEvent;

/// Application state for the web server
pub struct AppState {
    pub torrents: Arc<Mutex<Vec<TorrentInfo>>>,
    pub seeding_events: Arc<Mutex<Vec<SeedingEvent>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TorrentStats {
    pub total_torrents: usize,
    pub seeding: usize,
    pub downloading: usize,
    pub total_uploaded: u64,
    pub total_downloaded: u64,
}

/// Initialize web server routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health_check))
        .route("/api/torrents", get(get_torrents))
        .route("/api/torrents/stats", get(get_trrent_stats))
        .route("/api/torrents/prioritized", get(get_prioritized_torrents))
        .route("/api/seeding-events", get(get_seeding_events))
        .route("/api/seeding-events/recent", get(get_recent_seeding_events))
        .with_state(Arc::new(state))
        .layer(CorsLayer::permissive())
}

async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn get_torrents(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let torrents = state.torrents.lock().unwrap();
    Json(torrents.clone())
}

async fn get_trrent_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let torrents = state.torrents.lock().unwrap();

    let stats = TorrentStats {
        total_torrents: torrents.len(),
        seeding: torrents.iter().filter(|t| t.status == "Seeding").count(),
        downloading: torrents
            .iter()
            .filter(|t| t.status == "Downloading")
            .count(),
        total_uploaded: torrents.iter().map(|t| t.uploaded).sum(),
        total_downloaded: torrents.iter().map(|t| t.downloaded).sum(),
    };

    Json(stats)
}

async fn get_prioritized_torrents(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut torrents = state.torrents.lock().unwrap().clone();

    // Sort by fewest seeders first (prioritize seeding where needed most)
    torrents.sort_by_key(|t| std::cmp::Reverse(t.status.clone()));

    Json(torrents)
}

async fn get_seeding_events(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let events = state.seeding_events.lock().unwrap();
    Json(events.clone())
}

async fn get_recent_seeding_events(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let events = state.seeding_events.lock().unwrap();
    let recent: Vec<_> = events.iter().rev().take(100).cloned().collect();
    Json(recent)
}

/// Start the web server on the specified port
pub async fn start_web_server(port: u16, state: AppState) {
    let router = create_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind web server");

    tracing::info!("Web UI server listening on: http://0.0.0.0:{}", port);

    axum::serve(listener, router)
        .await
        .expect("Web server error");
}
