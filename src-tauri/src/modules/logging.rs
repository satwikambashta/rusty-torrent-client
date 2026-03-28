use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use std::path::PathBuf;
use anyhow::Result;

/// Initialize logging system with both console and file output
pub fn init_logging(log_dir: &PathBuf, verbose: bool) -> Result<()> {
    std::fs::create_dir_all(log_dir)?;

    let file_appender = tracing_appender::rolling::daily(log_dir, "torrent-client.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = if verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stdout)
        .with_target(true)
        .with_thread_ids(true);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .with(file_layer)
        .init();

    tracing::info!("Logging initialized successfully");
    Ok(())
}

/// Seeding event for logging
#[derive(Debug, Clone, serde::Serialize)]
pub struct SeedingEvent {
    pub timestamp: String,
    pub torrent_id: String,
    pub torrent_name: String,
    pub peer_ip: String,
    pub bytes_sent: u64,
    pub peer_seeders: u32,
    pub peer_leechers: u32,
}

impl SeedingEvent {
    /// Create a new seeding event
    pub fn new(
        torrent_id: String,
        torrent_name: String,
        peer_ip: String,
        bytes_sent: u64,
        peer_seeders: u32,
        peer_leechers: u32,
    ) -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            torrent_id,
            torrent_name,
            peer_ip,
            bytes_sent,
            peer_seeders,
            peer_leechers,
        }
    }

    /// Log this seeding event
    pub fn log(&self) {
        tracing::info!(
            torrent = %self.torrent_name,
            peer = %self.peer_ip,
            bytes = self.bytes_sent,
            seeders = self.peer_seeders,
            leechers = self.peer_leechers,
            "Seeding event"
        );
    }
}

/// Event log storage for visualization
pub struct EventLog {
    events: Vec<SeedingEvent>,
}

impl EventLog {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }

    pub fn add_event(&mut self, event: SeedingEvent) {
        event.log();
        self.events.push(event);
        
        // Keep only last 10000 events in memory
        if self.events.len() > 10000 {
            self.events.drain(0..5000);
        }
    }

    pub fn get_events(&self, limit: usize) -> Vec<SeedingEvent> {
        self.events.iter().rev().take(limit).cloned().collect()
    }
}
