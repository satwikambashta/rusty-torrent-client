// Remote API types for web client

export interface TorrentInfo {
  id: string;
  name: string;
  total_size: number;
  downloaded: number;
  uploaded: number;
  status: "Idle" | "Downloading" | "Uploading" | "Seeding" | "Paused" | "Error";
  progress: number; // 0-100
}

export interface TorrentStats {
  total_torrents: number;
  seeding: number;
  downloading: number;
  total_uploaded: number;
  total_downloaded: number;
}

export interface HealthResponse {
  status: string;
  version: string;
}

export interface SeedingEvent {
  timestamp: string;
  torrent_id: string;
  torrent_name: string;
  peer_ip: string;
  bytes_sent: number;
  peer_seeders: number;
  peer_leechers: number;
}

export interface ApiError {
  message: string;
  status: number;
}
